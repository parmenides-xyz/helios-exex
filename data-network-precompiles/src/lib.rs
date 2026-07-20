//! DATA Network precompile implementations.

use std::{fmt::Display, iter};

use alloy_primitives::{Address, Bytes};
use revm::{
    context::{Cfg, ContextTr},
    handler::{ContextTrDbError, EthPrecompiles, PrecompileProvider},
    interpreter::{CallInputs, CallScheme, Gas, InstructionResult, InterpreterResult},
    precompile::{PrecompileError, PrecompileResult},
    primitives::hardfork::SpecId,
};

pub mod error;
pub mod ip_graph;
pub mod storage;

pub use error::{DataNetworkPrecompileError, Result};

use ip_graph::{IpGraph, IP_GRAPH_ADDRESS};
use storage::{evm::EvmPrecompileStorageProvider, StorageCtx};

/// Trait implemented by DATA Network precompile contract types.
pub trait Precompile {
    /// ABI-decodes calldata and dispatches it to the matching precompile method.
    fn call(
        &mut self,
        calldata: &[u8],
        msg_sender: Address,
        call_scheme: CallScheme,
    ) -> PrecompileResult;
}

/// Ethereum precompiles extended with DATA Network's stateful precompiles.
#[derive(Debug, Clone, Default)]
pub struct DataNetworkPrecompiles {
    inner: EthPrecompiles,
}

impl<CTX> PrecompileProvider<CTX> for DataNetworkPrecompiles
where
    CTX: ContextTr<Cfg: Cfg<Spec = SpecId>>,
    ContextTrDbError<CTX>: Display,
{
    type Output = InterpreterResult;

    fn set_spec(&mut self, spec: <CTX::Cfg as Cfg>::Spec) -> bool {
        <EthPrecompiles as PrecompileProvider<CTX>>::set_spec(&mut self.inner, spec)
    }

    fn run(
        &mut self,
        context: &mut CTX,
        inputs: &CallInputs,
    ) -> std::result::Result<Option<Self::Output>, String> {
        if inputs.bytecode_address != IP_GRAPH_ADDRESS {
            return <EthPrecompiles as PrecompileProvider<CTX>>::run(
                &mut self.inner,
                context,
                inputs,
            );
        }

        let calldata = inputs.input.bytes(context);
        let required_gas = IpGraph::default().required_gas(&calldata);

        let result = if required_gas > inputs.gas_limit {
            Err(PrecompileError::OutOfGas)
        } else {
            let mut storage = EvmPrecompileStorageProvider::new(context, inputs.is_static);
            StorageCtx::enter(&mut storage, || {
                IpGraph::default().call(&calldata, inputs.caller, inputs.scheme)
            })
            .map(|mut output| {
                output.gas_used = required_gas;
                output
            })
        };

        precompile_result_to_interpreter(result, inputs.gas_limit).map(Some)
    }

    fn warm_addresses(&self) -> Box<impl Iterator<Item = Address>> {
        Box::new(
            self.inner
                .warm_addresses()
                .chain(iter::once(IP_GRAPH_ADDRESS)),
        )
    }

    fn contains(&self, address: &Address) -> bool {
        *address == IP_GRAPH_ADDRESS || self.inner.contains(address)
    }
}

fn precompile_result_to_interpreter(
    result: PrecompileResult,
    gas_limit: u64,
) -> std::result::Result<InterpreterResult, String> {
    let mut interpreter_result = InterpreterResult {
        result: InstructionResult::Return,
        gas: Gas::new(gas_limit),
        output: Bytes::new(),
    };

    match result {
        Ok(output) => {
            let recorded = interpreter_result.gas.record_cost(output.gas_used);
            assert!(recorded, "precompile gas was checked before execution");
            interpreter_result.result = if output.reverted {
                InstructionResult::Revert
            } else {
                InstructionResult::Return
            };
            interpreter_result.output = output.bytes;
        }
        Err(PrecompileError::Fatal(error)) => return Err(error),
        Err(error) => {
            interpreter_result.result = if error.is_oog() {
                InstructionResult::PrecompileOOG
            } else {
                InstructionResult::PrecompileError
            };
        }
    }

    Ok(interpreter_result)
}
