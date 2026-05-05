//! WASM virtual machine for smart contract execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasmi::*;

/// Error types for VM execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmError {
    /// Failed to compile WASM module
    InvalidModule(String),
    /// Contract execution ran out of gas
    OutOfGas,
    /// Execution failed with a message
    ExecutionFailed(String),
    /// Host function error
    HostError(String),
    /// Memory access error
    MemoryError(String),
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmError::InvalidModule(msg) => write!(f, "Invalid WASM module: {}", msg),
            VmError::OutOfGas => write!(f, "Out of gas"),
            VmError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            VmError::HostError(msg) => write!(f, "Host error: {}", msg),
            VmError::MemoryError(msg) => write!(f, "Memory error: {}", msg),
        }
    }
}

/// Event emitted by a smart contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    pub topic: String,
    pub data: Vec<u8>,
}

/// Result of contract execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmResult {
    /// Data returned by the contract function
    pub return_data: Vec<u8>,
    /// Total gas consumed
    pub gas_used: u64,
    /// Events emitted during execution
    pub events: Vec<ContractEvent>,
    /// Storage changes: key -> Some(value) for writes, None for deletes
    pub storage_changes: HashMap<Vec<u8>, Option<Vec<u8>>>,
}

/// Context for contract execution.
#[derive(Debug, Clone)]
pub struct VmContext {
    /// Address of the caller
    pub caller: String,
    /// Address of the contract being called
    pub contract_address: String,
    /// Remaining gas
    pub gas_remaining: u64,
    /// Current block height
    pub block_height: u64,
    /// Current contract storage snapshot
    pub storage: HashMap<Vec<u8>, Vec<u8>>,
    /// Account balances (for get_balance host function)
    pub balances: HashMap<String, u64>,
    /// Amount of native tokens sent with the call
    pub value: u64,
}

/// Mutable state tracked during execution.
struct ExecutionState {
    gas_remaining: u64,
    storage_changes: HashMap<Vec<u8>, Option<Vec<u8>>>,
    events: Vec<ContractEvent>,
    return_data: Vec<u8>,
}

/// Execute a WASM contract.
///
/// # Arguments
/// * `wasm_code` - The WASM bytecode
/// * `function` - The exported function name to call
/// * `args` - Encoded arguments (as raw bytes, interpreted as i64 values)
/// * `context` - Execution context
///
/// # Returns
/// `VmResult` with return data, gas used, events, and storage changes.
pub fn execute(
    wasm_code: &[u8],
    function: &str,
    args: &[u8],
    context: &VmContext,
) -> Result<VmResult, VmError> {
    // Create the WASM engine with fuel consumption enabled
    let mut config = Config::default();
    config.consume_fuel(true);
    let engine = Engine::new(&config);

    // Compile the module
    let module =
        Module::new(&engine, wasm_code).map_err(|e| VmError::InvalidModule(e.to_string()))?;

    // Create a store with execution state
    let mut state = ExecutionState {
        gas_remaining: context.gas_remaining,
        storage_changes: HashMap::new(),
        events: Vec::new(),
        return_data: Vec::new(),
    };

    // Set fuel for gas metering
    let mut store = Store::new(&engine, state);
    store
        .set_fuel(context.gas_remaining as u64)
        .map_err(|e| VmError::HostError(e.to_string()))?;

    // Create a linker for host functions
    let mut linker = <Linker<ExecutionState>>::new(&engine);

    // Register host functions
    register_host_functions(&mut linker, context).map_err(|e| VmError::HostError(e.to_string()))?;

    // Instantiate the module
    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(|e| VmError::ExecutionFailed(e.to_string()))?
        .start(&mut store)
        .map_err(|e| VmError::ExecutionFailed(e.to_string()))?;

    // Get the exported function
    let func = instance
        .get_func(&store, function)
        .ok_or_else(|| VmError::ExecutionFailed(format!("Function '{}' not found", function)))?;

    // Prepare arguments: decode args bytes as i64 values
    let wasm_args: Vec<Val> = args
        .chunks_exact(8)
        .map(|chunk| {
            let val = i64::from_le_bytes(chunk.try_into().unwrap_or([0; 8]));
            Val::I64(val)
        })
        .collect();

    // Call the function
    let mut results = vec![Val::I64(0)];
    func.call(&mut store, &wasm_args, &mut results)
        .map_err(|e| VmError::ExecutionFailed(e.to_string()))?;

    // Calculate gas used
    let fuel_remaining = store.get_fuel().unwrap_or(0);
    let gas_used = context.gas_remaining.saturating_sub(fuel_remaining as u64);

    // Extract state from store
    let exec_state = store.into_data();

    Ok(VmResult {
        return_data: exec_state.return_data,
        gas_used,
        events: exec_state.events,
        storage_changes: exec_state.storage_changes,
    })
}

/// Register host functions that contracts can call.
fn register_host_functions(
    linker: &mut Linker<ExecutionState>,
    context: &VmContext,
) -> Result<(), wasmi::Error> {
    let caller_addr = context.caller.clone();
    let contract_addr = context.contract_address.clone();
    let block_height = context.block_height;
    let storage_snapshot = context.storage.clone();
    let balances = context.balances.clone();

    // storage_read(key_ptr, key_len, val_ptr, val_len) -> i32
    // Returns the number of bytes written to val_ptr, or -1 if key not found.
    let storage_for_read = storage_snapshot.clone();
    linker.func_wrap(
        "env",
        "storage_read",
        move |mut caller: Caller<'_, ExecutionState>,
              key_ptr: i32,
              key_len: i32,
              val_ptr: i32,
              val_len: i32|
              -> i32 {
            // Read the key from WASM memory
            let memory = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return -1,
            };
            let data = caller.data();
            let gas_cost = 100; // storage_read_cost
            if data.gas_remaining < gas_cost {
                return -1;
            }

            let mut key_buf = vec![0u8; key_len as usize];
            if memory
                .read(&caller, key_ptr as usize, &mut key_buf)
                .is_err()
            {
                return -1;
            }

            // Check storage changes first, then snapshot
            let value = {
                let data = caller.data();
                if let Some(Some(v)) = data.storage_changes.get(&key_buf) {
                    Some(v.clone())
                } else if data.storage_changes.contains_key(&key_buf) {
                    None // deleted
                } else {
                    storage_for_read.get(&key_buf).cloned()
                }
            };

            match value {
                Some(val) => {
                    let copy_len = val.len().min(val_len as usize);
                    if memory
                        .write(&mut caller, val_ptr as usize, &val[..copy_len])
                        .is_err()
                    {
                        return -1;
                    }
                    caller.data_mut().gas_remaining =
                        caller.data_mut().gas_remaining.saturating_sub(gas_cost);
                    copy_len as i32
                }
                None => -1,
            }
        },
    )?;

    // storage_write(key_ptr, key_len, val_ptr, val_len)
    linker.func_wrap(
        "env",
        "storage_write",
        move |mut caller: Caller<'_, ExecutionState>,
              key_ptr: i32,
              key_len: i32,
              val_ptr: i32,
              val_len: i32| {
            let memory = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return,
            };

            let gas_cost = 500; // storage_write_cost
            if caller.data().gas_remaining < gas_cost {
                return;
            }

            let mut key_buf = vec![0u8; key_len as usize];
            let mut val_buf = vec![0u8; val_len as usize];
            if memory
                .read(&caller, key_ptr as usize, &mut key_buf)
                .is_err()
            {
                return;
            }
            if memory
                .read(&caller, val_ptr as usize, &mut val_buf)
                .is_err()
            {
                return;
            }

            caller
                .data_mut()
                .storage_changes
                .insert(key_buf, Some(val_buf));
            caller.data_mut().gas_remaining =
                caller.data_mut().gas_remaining.saturating_sub(gas_cost);
        },
    )?;

    // storage_delete(key_ptr, key_len)
    linker.func_wrap(
        "env",
        "storage_delete",
        move |mut caller: Caller<'_, ExecutionState>, key_ptr: i32, key_len: i32| {
            let memory = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return,
            };

            let gas_cost = 500;
            if caller.data().gas_remaining < gas_cost {
                return;
            }

            let mut key_buf = vec![0u8; key_len as usize];
            if memory
                .read(&caller, key_ptr as usize, &mut key_buf)
                .is_err()
            {
                return;
            }

            caller.data_mut().storage_changes.insert(key_buf, None);
            caller.data_mut().gas_remaining =
                caller.data_mut().gas_remaining.saturating_sub(gas_cost);
        },
    )?;

    // get_caller(ptr, max_len) -> i32 (bytes written)
    let caller_clone = caller_addr.clone();
    linker.func_wrap(
        "env",
        "get_caller",
        move |mut caller: Caller<'_, ExecutionState>, ptr: i32, max_len: i32| -> i32 {
            let memory = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return 0,
            };
            let addr = caller_clone.as_bytes();
            let copy_len = addr.len().min(max_len as usize);
            if memory
                .write(&mut caller, ptr as usize, &addr[..copy_len])
                .is_err()
            {
                return 0;
            }
            copy_len as i32
        },
    )?;

    // get_block_height() -> i64
    linker.func_wrap("env", "get_block_height", move || -> i64 {
        block_height as i64
    })?;

    // get_balance(addr_ptr, addr_len) -> i64
    let balances_clone = balances.clone();
    linker.func_wrap(
        "env",
        "get_balance",
        move |mut caller: Caller<'_, ExecutionState>, addr_ptr: i32, addr_len: i32| -> i64 {
            let memory = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return 0,
            };
            let mut addr_buf = vec![0u8; addr_len as usize];
            if memory
                .read(&caller, addr_ptr as usize, &mut addr_buf)
                .is_err()
            {
                return 0;
            }
            let addr = String::from_utf8_lossy(&addr_buf).to_string();
            balances_clone.get(&addr).copied().unwrap_or(0) as i64
        },
    )?;

    // get_contract_address(ptr, max_len) -> i32
    let contract_clone = contract_addr.clone();
    linker.func_wrap(
        "env",
        "get_contract_address",
        move |mut caller: Caller<'_, ExecutionState>, ptr: i32, max_len: i32| -> i32 {
            let memory = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return 0,
            };
            let addr = contract_clone.as_bytes();
            let copy_len = addr.len().min(max_len as usize);
            if memory
                .write(&mut caller, ptr as usize, &addr[..copy_len])
                .is_err()
            {
                return 0;
            }
            copy_len as i32
        },
    )?;

    // get_value() -> i64 (amount of native tokens sent with the call)
    let value = context.value;
    linker.func_wrap("env", "get_value", move || -> i64 { value as i64 })?;

    // emit_event(topic_ptr, topic_len, data_ptr, data_len)
    linker.func_wrap(
        "env",
        "emit_event",
        move |mut caller: Caller<'_, ExecutionState>,
              topic_ptr: i32,
              topic_len: i32,
              data_ptr: i32,
              data_len: i32| {
            let memory = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return,
            };

            let gas_cost = 200;
            if caller.data().gas_remaining < gas_cost {
                return;
            }

            let mut topic_buf = vec![0u8; topic_len as usize];
            let mut data_buf = vec![0u8; data_len as usize];
            if memory
                .read(&caller, topic_ptr as usize, &mut topic_buf)
                .is_err()
            {
                return;
            }
            if memory
                .read(&caller, data_ptr as usize, &mut data_buf)
                .is_err()
            {
                return;
            }

            let topic = String::from_utf8_lossy(&topic_buf).to_string();
            caller.data_mut().events.push(ContractEvent {
                topic,
                data: data_buf,
            });
            caller.data_mut().gas_remaining =
                caller.data_mut().gas_remaining.saturating_sub(gas_cost);
        },
    )?;

    // return_data(ptr, len)
    linker.func_wrap(
        "env",
        "set_return_data",
        move |mut caller: Caller<'_, ExecutionState>, ptr: i32, len: i32| {
            let memory = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return,
            };
            let mut buf = vec![0u8; len as usize];
            if memory.read(&caller, ptr as usize, &mut buf).is_err() {
                return;
            }
            caller.data_mut().return_data = buf;
        },
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context(caller: &str, contract: &str, gas: u64) -> VmContext {
        VmContext {
            caller: caller.to_string(),
            contract_address: contract.to_string(),
            gas_remaining: gas,
            block_height: 1,
            storage: HashMap::new(),
            balances: HashMap::new(),
            value: 0,
        }
    }

    #[test]
    fn test_invalid_wasm_module() {
        let ctx = make_context("alice", "contract1", 100_000);
        let result = execute(&[0x00, 0x01, 0x02], "main", &[], &ctx);
        assert!(matches!(result, Err(VmError::InvalidModule(_))));
    }

    #[test]
    fn test_function_not_found() {
        // Minimal valid WASM module (empty)
        let wasm = wat::parse_str("(module)").unwrap();
        let ctx = make_context("alice", "contract1", 100_000);
        let result = execute(&wasm, "nonexistent", &[], &ctx);
        assert!(matches!(result, Err(VmError::ExecutionFailed(_))));
    }

    #[test]
    fn test_simple_function_call() {
        // A WASM module that exports an "add" function taking two i64 args and returning their sum
        let wat = r#"
            (module
                (func (export "add") (param i64 i64) (result i64)
                    local.get 0
                    local.get 1
                    i64.add
                )
            )
        "#;
        let wasm = wat::parse_str(wat).unwrap();
        let ctx = make_context("alice", "contract1", 100_000);

        // Encode two i64 arguments: 3 and 5
        let mut args = Vec::new();
        args.extend_from_slice(&3i64.to_le_bytes());
        args.extend_from_slice(&5i64.to_le_bytes());

        let result = execute(&wasm, "add", &args, &ctx).unwrap();
        assert_eq!(result.gas_used > 0, true);
        // The return value is in the results, not return_data
    }
}
