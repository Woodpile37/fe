use std::fmt;

use fe_analyzer::namespace::items::ContractId;
use id_arena::Id;

use super::{basic_block::BasicBlockId, function::FunctionId, value::ValueId, SourceInfo, TypeId};

pub type InstId = Id<Inst>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Inst {
    pub kind: InstKind,
    pub source: SourceInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstKind {
    /// This is not a real instruction, just used to tag a position where a
    /// local is declared.
    Declare {
        local: ValueId,
    },

    Assign {
        lhs: ValueId,
        rhs: ValueId,
    },

    /// Unary instruction.
    Unary {
        op: UnOp,
        value: ValueId,
    },

    /// Binary instruction.
    Binary {
        op: BinOp,
        lhs: ValueId,
        rhs: ValueId,
    },

    Cast {
        value: ValueId,
        to: TypeId,
    },

    /// Constructs aggregate value, i.e. struct, tuple and array.
    AggregateConstruct {
        ty: TypeId,
        args: Vec<ValueId>,
    },

    /// Access to aggregate fields or elements.
    /// # Example
    ///
    /// ```fe
    /// struct Foo:
    ///     x: i32
    ///     y: Array<i32, 8>
    /// ```
    /// `foo.y` is lowered into `AggregateAccess(foo, [1])' for example.
    AggregateAccess {
        value: ValueId,
        index: ValueId,
    },

    MapAccess {
        value: ValueId,
        key: ValueId,
    },

    Call {
        func: FunctionId,
        args: Vec<ValueId>,
        call_type: CallType,
    },

    /// Unconditional jump instruction.
    Jump {
        dest: BasicBlockId,
    },

    /// Conditional branching instruction.
    Branch {
        cond: ValueId,
        then: BasicBlockId,
        else_: BasicBlockId,
    },

    Revert {
        arg: ValueId,
    },

    Emit {
        arg: ValueId,
    },

    Return {
        arg: ValueId,
    },

    Keccak256 {
        arg: ValueId,
    },

    Clone {
        arg: ValueId,
    },

    ToMem {
        arg: ValueId,
    },

    AbiEncode {
        arg: ValueId,
    },

    Create {
        value: ValueId,
        contract: ContractId,
    },

    Create2 {
        value: ValueId,
        salt: ValueId,
        contract: ContractId,
    },

    YulIntrinsic {
        op: YulIntrinsicOp,
        args: Vec<ValueId>,
    },
}

impl Inst {
    pub fn new(kind: InstKind, source: SourceInfo) -> Self {
        Self { kind, source }
    }

    pub fn unary(op: UnOp, value: ValueId, source: SourceInfo) -> Self {
        let kind = InstKind::Unary { op, value };
        Self::new(kind, source)
    }

    pub fn binary(op: BinOp, lhs: ValueId, rhs: ValueId, source: SourceInfo) -> Self {
        let kind = InstKind::Binary { op, lhs, rhs };
        Self::new(kind, source)
    }

    pub fn intrinsic(op: YulIntrinsicOp, args: Vec<ValueId>, source: SourceInfo) -> Self {
        let kind = InstKind::YulIntrinsic { op, args };
        Self::new(kind, source)
    }

    pub fn is_terminator(&self) -> bool {
        match self.kind {
            InstKind::Jump { .. }
            | InstKind::Branch { .. }
            | InstKind::Revert { .. }
            | InstKind::Return { .. } => true,
            InstKind::YulIntrinsic { op, .. } => op.is_terminator(),
            _ => false,
        }
    }

    pub fn branch_info(&self) -> BranchInfo {
        match self.kind {
            InstKind::Jump { dest } => BranchInfo::Jump(dest),
            InstKind::Branch { then, else_, .. } => BranchInfo::Branch((then, else_)),
            _ => BranchInfo::NotBranch,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnOp {
    /// `not` operator for logical inversion.
    Not,
    /// `-` operator for negation.
    Neg,
    /// `~` operator for bitwise inversion.
    Inv,
}

impl fmt::Display for UnOp {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Not => write!(w, "not"),
            Self::Neg => write!(w, "-"),
            Self::Inv => write!(w, "~"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Shl,
    Shr,
    BitOr,
    BitXor,
    BitAnd,
    LogicalAnd,
    LogicalOr,
    Eq,
    Ne,
    Ge,
    Gt,
    Le,
    Lt,
}

impl fmt::Display for BinOp {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Add => write!(w, "+"),
            Self::Sub => write!(w, "-"),
            Self::Mul => write!(w, "*"),
            Self::Div => write!(w, "/"),
            Self::Mod => write!(w, "%"),
            Self::Pow => write!(w, "**"),
            Self::Shl => write!(w, "<<"),
            Self::Shr => write!(w, ">>"),
            Self::BitOr => write!(w, "|"),
            Self::BitXor => write!(w, "^"),
            Self::BitAnd => write!(w, "&"),
            Self::LogicalAnd => write!(w, "and"),
            Self::LogicalOr => write!(w, "or"),
            Self::Eq => write!(w, "=="),
            Self::Ne => write!(w, "!="),
            Self::Ge => write!(w, ">="),
            Self::Gt => write!(w, ">"),
            Self::Le => write!(w, "<="),
            Self::Lt => write!(w, "<"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallType {
    Internal,
    External,
}

impl fmt::Display for CallType {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Internal => write!(w, "internal"),
            Self::External => write!(w, "external"),
        }
    }
}

// TODO: We don't need all yul intrinsics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum YulIntrinsicOp {
    Stop,
    Add,
    Sub,
    Mul,
    Div,
    Sdiv,
    Mod,
    Smod,
    Exp,
    Not,
    Lt,
    Gt,
    Slt,
    Sgt,
    Eq,
    Iszero,
    And,
    Or,
    Xor,
    Byte,
    Shl,
    Shr,
    Sar,
    Addmod,
    Mulmod,
    Signextend,
    Keccak256,
    Pc,
    Pop,
    Mload,
    Mstore,
    Mstore8,
    Sload,
    Sstore,
    Msize,
    Gas,
    Address,
    Balance,
    Selfbalance,
    Caller,
    Callvalue,
    Calldataload,
    Calldatasize,
    Calldatacopy,
    Codesize,
    Codecopy,
    Extcodesize,
    Extcodecopy,
    Returndatasize,
    Returndatacopy,
    Extcodehash,
    Create,
    Create2,
    Call,
    Callcode,
    Delegatecall,
    Staticcall,
    Return,
    Revert,
    Selfdestruct,
    Invalid,
    Log0,
    Log1,
    Log2,
    Log3,
    Log4,
    Chainid,
    Basefee,
    Origin,
    Gasprice,
    Blockhash,
    Coinbase,
    Timestamp,
    Number,
    Difficulty,
    Gaslimit,
}
impl YulIntrinsicOp {
    pub fn is_terminator(self) -> bool {
        matches!(
            self,
            Self::Return | Self::Revert | Self::Selfdestruct | Self::Invalid
        )
    }
}

impl fmt::Display for YulIntrinsicOp {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        let op = match self {
            Self::Stop => "__stop",
            Self::Add => "__add",
            Self::Sub => "__sub",
            Self::Mul => "__mul",
            Self::Div => "__div",
            Self::Sdiv => "__sdiv",
            Self::Mod => "__mod",
            Self::Smod => "__smod",
            Self::Exp => "__exp",
            Self::Not => "__not",
            Self::Lt => "__lt",
            Self::Gt => "__gt",
            Self::Slt => "__slt",
            Self::Sgt => "__sgt",
            Self::Eq => "__eq",
            Self::Iszero => "__iszero",
            Self::And => "__and",
            Self::Or => "__or",
            Self::Xor => "__xor",
            Self::Byte => "__byte",
            Self::Shl => "__shl",
            Self::Shr => "__shr",
            Self::Sar => "__sar",
            Self::Addmod => "__addmod",
            Self::Mulmod => "__mulmod",
            Self::Signextend => "__signextend",
            Self::Keccak256 => "__keccak256",
            Self::Pc => "__pc",
            Self::Pop => "__pop",
            Self::Mload => "__mload",
            Self::Mstore => "__mstore",
            Self::Mstore8 => "__mstore8",
            Self::Sload => "__sload",
            Self::Sstore => "__sstore",
            Self::Msize => "__msize",
            Self::Gas => "__gas",
            Self::Address => "__address",
            Self::Balance => "__balance",
            Self::Selfbalance => "__selfbalance",
            Self::Caller => "__caller",
            Self::Callvalue => "__callvalue",
            Self::Calldataload => "__calldataload",
            Self::Calldatasize => "__calldatasize",
            Self::Calldatacopy => "__calldatacopy",
            Self::Codesize => "__codesize",
            Self::Codecopy => "__codecopy",
            Self::Extcodesize => "__extcodesize",
            Self::Extcodecopy => "__extcodecopy",
            Self::Returndatasize => "__returndatasize",
            Self::Returndatacopy => "__returndatacopy",
            Self::Extcodehash => "__extcodehash",
            Self::Create => "__create",
            Self::Create2 => "__create2",
            Self::Call => "__call",
            Self::Callcode => "__callcode",
            Self::Delegatecall => "__delegatecall",
            Self::Staticcall => "__staticcall",
            Self::Return => "__return",
            Self::Revert => "__revert",
            Self::Selfdestruct => "__selfdestruct",
            Self::Invalid => "__invalid",
            Self::Log0 => "__log0",
            Self::Log1 => "__log1",
            Self::Log2 => "__log2",
            Self::Log3 => "__log3",
            Self::Log4 => "__log4",
            Self::Chainid => "__chainid",
            Self::Basefee => "__basefee",
            Self::Origin => "__origin",
            Self::Gasprice => "__gasprice",
            Self::Blockhash => "__blockhash",
            Self::Coinbase => "__coinbase",
            Self::Timestamp => "__timestamp",
            Self::Number => "__number",
            Self::Difficulty => "__difficulty",
            Self::Gaslimit => "__gaslimit",
        };

        write!(w, "{}", op)
    }
}

impl From<fe_analyzer::builtins::Intrinsic> for YulIntrinsicOp {
    fn from(val: fe_analyzer::builtins::Intrinsic) -> Self {
        use fe_analyzer::builtins::Intrinsic;
        match val {
            Intrinsic::__stop => Self::Stop,
            Intrinsic::__add => Self::Add,
            Intrinsic::__sub => Self::Sub,
            Intrinsic::__mul => Self::Mul,
            Intrinsic::__div => Self::Div,
            Intrinsic::__sdiv => Self::Sdiv,
            Intrinsic::__mod => Self::Mod,
            Intrinsic::__smod => Self::Smod,
            Intrinsic::__exp => Self::Exp,
            Intrinsic::__not => Self::Not,
            Intrinsic::__lt => Self::Lt,
            Intrinsic::__gt => Self::Gt,
            Intrinsic::__slt => Self::Slt,
            Intrinsic::__sgt => Self::Sgt,
            Intrinsic::__eq => Self::Eq,
            Intrinsic::__iszero => Self::Iszero,
            Intrinsic::__and => Self::And,
            Intrinsic::__or => Self::Or,
            Intrinsic::__xor => Self::Xor,
            Intrinsic::__byte => Self::Byte,
            Intrinsic::__shl => Self::Shl,
            Intrinsic::__shr => Self::Shr,
            Intrinsic::__sar => Self::Sar,
            Intrinsic::__addmod => Self::Addmod,
            Intrinsic::__mulmod => Self::Mulmod,
            Intrinsic::__signextend => Self::Signextend,
            Intrinsic::__keccak256 => Self::Keccak256,
            Intrinsic::__pc => Self::Pc,
            Intrinsic::__pop => Self::Pop,
            Intrinsic::__mload => Self::Mload,
            Intrinsic::__mstore => Self::Mstore,
            Intrinsic::__mstore8 => Self::Mstore8,
            Intrinsic::__sload => Self::Sload,
            Intrinsic::__sstore => Self::Sstore,
            Intrinsic::__msize => Self::Msize,
            Intrinsic::__gas => Self::Gas,
            Intrinsic::__address => Self::Address,
            Intrinsic::__balance => Self::Balance,
            Intrinsic::__selfbalance => Self::Selfbalance,
            Intrinsic::__caller => Self::Caller,
            Intrinsic::__callvalue => Self::Callvalue,
            Intrinsic::__calldataload => Self::Calldataload,
            Intrinsic::__calldatasize => Self::Calldatasize,
            Intrinsic::__calldatacopy => Self::Calldatacopy,
            Intrinsic::__codesize => Self::Codesize,
            Intrinsic::__codecopy => Self::Codecopy,
            Intrinsic::__extcodesize => Self::Extcodesize,
            Intrinsic::__extcodecopy => Self::Extcodecopy,
            Intrinsic::__returndatasize => Self::Returndatasize,
            Intrinsic::__returndatacopy => Self::Returndatacopy,
            Intrinsic::__extcodehash => Self::Extcodehash,
            Intrinsic::__create => Self::Create,
            Intrinsic::__create2 => Self::Create2,
            Intrinsic::__call => Self::Call,
            Intrinsic::__callcode => Self::Callcode,
            Intrinsic::__delegatecall => Self::Delegatecall,
            Intrinsic::__staticcall => Self::Staticcall,
            Intrinsic::__return => Self::Return,
            Intrinsic::__revert => Self::Revert,
            Intrinsic::__selfdestruct => Self::Selfdestruct,
            Intrinsic::__invalid => Self::Invalid,
            Intrinsic::__log0 => Self::Log0,
            Intrinsic::__log1 => Self::Log1,
            Intrinsic::__log2 => Self::Log2,
            Intrinsic::__log3 => Self::Log3,
            Intrinsic::__log4 => Self::Log4,
            Intrinsic::__chainid => Self::Chainid,
            Intrinsic::__basefee => Self::Basefee,
            Intrinsic::__origin => Self::Origin,
            Intrinsic::__gasprice => Self::Gasprice,
            Intrinsic::__blockhash => Self::Blockhash,
            Intrinsic::__coinbase => Self::Coinbase,
            Intrinsic::__timestamp => Self::Timestamp,
            Intrinsic::__number => Self::Number,
            Intrinsic::__difficulty => Self::Difficulty,
            Intrinsic::__gaslimit => Self::Gaslimit,
        }
    }
}

pub enum BranchInfo {
    NotBranch,
    Jump(BasicBlockId),
    Branch((BasicBlockId, BasicBlockId)),
}
