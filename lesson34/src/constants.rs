pub const MAGIC_DATA_SIZE: usize = 2;
pub const MAX_ACCOUNT_ID_LEN: usize = 16;

pub type TypeIdMark = u8;
//ser-de base types
pub const TYPE_ID_ACCOUNT_ID: TypeIdMark = 42;
pub const TYPE_ID_MONEY: TypeIdMark = 52;
pub const TYPE_ID_NONZERO_MONEY: TypeIdMark = 62;
pub const TYPE_ID_RESULT_OK: TypeIdMark = 72;
pub const TYPE_ID_RESULT_ERR: TypeIdMark = 82;
pub const TYPE_ID_ACCOUNT: TypeIdMark = 92;
pub const TYPE_ID_VEC: TypeIdMark = 102;
//ser-de Operation
pub const TYPE_ID_OPERATION_CREATE: TypeIdMark = 1;
pub const TYPE_ID_OPERATION_DEPOSIT: TypeIdMark = 2;
pub const TYPE_ID_OPERATION_WITHDRAW: TypeIdMark = 3;
pub const TYPE_ID_OPERATION_MOVE: TypeIdMark = 4;
pub const TYPE_ID_OPERATION_GETBALANCE: TypeIdMark = 5;
//ser-de Protocol
pub const TYPE_ID_PROTOCOL_REQUEST: TypeIdMark = 15;
pub const TYPE_ID_PROTOCOL_RESPONSE: TypeIdMark = 16;
