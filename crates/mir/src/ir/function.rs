use fe_common::impl_intern_key;
use fxhash::FxHashMap;
use id_arena::Arena;
use smol_str::SmolStr;

use super::{
    basic_block::BasicBlock,
    body_order::BodyOrder,
    inst::{Inst, InstId},
    module::ModuleId,
    types::{Type, TypeId},
    value::{Immediate, Value, ValueId},
    SourceInfo,
};

/// Represents function signature.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionSignature {
    params: Vec<FunctionParam>,
    return_type: Type,
    module_id: ModuleId,
    linkage: Linkage,
    source: SourceInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionParam {
    name: SmolStr,
    ty: Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(u32);
impl_intern_key!(FunctionId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Linkage {
    /// A function can only be called within the same module.
    Private,

    /// A function can be called from other modules, but can NOT be called from
    /// other accounts and transactions.
    Public,

    /// A function can be called from other modules, and also can be called from
    /// other accounts and transactions.
    Export,
}

/// A function body, which is not stored in salsa db to enable in-place
/// transformation.
pub struct FunctionBody {
    pub function: FunctionId,

    pub basic_blocks: Arena<BasicBlock>,

    pub body_data: BodyDataStore,

    /// All declared local variables in a function.
    pub locals: Vec<ValueId>,

    /// Tracks order of basic blocks and instructions in a function body.
    pub order: BodyOrder,

    pub source: SourceInfo,
}

/// A collection of instructions and values appear in a function body.
/// This struct also tracks data flow
pub struct BodyDataStore {
    /// Instructions appear in a function body.
    insts: Arena<Inst>,

    /// All values in a function.
    values: Arena<Value>,

    /// Map an immediate to a value to ensure the same immediate results in the
    /// same value.
    immediates: FxHashMap<Immediate, ValueId>,

    unit_value: Option<ValueId>,
}

impl BodyDataStore {
    pub fn store_inst(&mut self, inst: Inst) -> InstId {
        self.insts.alloc(inst)
    }

    /// Store `Value` and returns its identifier.
    pub fn store_value(&mut self, value: Value) -> ValueId {
        match value {
            Value::Immediate(imm) => self.store_immediate(imm),

            Value::Unit(_) => {
                if let Some(unit_value) = self.unit_value {
                    unit_value
                } else {
                    let unit_value = self.values.alloc(value);
                    self.unit_value = Some(unit_value);
                    unit_value
                }
            }

            _ => self.values.alloc(value),
        }
    }

    /// Returns an instruction result. A returned value is guaranteed to be a
    /// temporary value.
    pub fn inst_result(&mut self, inst: InstId) -> Option<ValueId> {
        self.insts[inst].kind.inst_result()
    }

    pub fn value_ty(&self, vid: ValueId) -> TypeId {
        self.values[vid].ty()
    }

    fn store_immediate(&mut self, imm: Immediate) -> ValueId {
        if let Some(value) = self.immediates.get(&imm) {
            *value
        } else {
            let id = self.values.alloc(Value::Immediate(imm.clone()));
            self.immediates.insert(imm, id);
            id
        }
    }
}