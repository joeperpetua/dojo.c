use std::ffi::{c_char, CStr, CString};

use torii_client::client::Client as TClient;

pub struct ToriiClient(pub TClient);

#[derive(Clone)]
#[repr(C)]
pub struct CArray<T> {
    data: *const T,
    data_len: usize,
}

#[repr(C)]
pub struct Error {
    pub message: *const c_char,
}

#[derive(Clone)]
#[repr(C)]
pub struct FieldElement {
    data: [u8; 32],
}

impl From<&FieldElement> for starknet::core::types::FieldElement {
    fn from(val: &FieldElement) -> Self {
        starknet::core::types::FieldElement::from_bytes_be(&val.data).unwrap()
    }
}

impl From<&starknet::core::types::FieldElement> for FieldElement {
    fn from(val: &starknet::core::types::FieldElement) -> Self {
        FieldElement {
            data: val.to_bytes_be(),
        }
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct EntityQuery {
    pub model: *const c_char,
    pub clause: Clause,
}

#[derive(Clone)]
#[repr(C)]
pub enum Clause {
    Keys(KeysClause),
    Attribute(AttributeClause),
    Composite(CompositeClause),
}

pub type KeysClause = CArray<FieldElement>;

#[derive(Clone)]
#[repr(C)]
pub struct AttributeClause {
    pub attribute: *const c_char,
    pub operator: ComparisonOperator,
    pub value: Value,
}

#[derive(Clone)]
#[repr(C)]
pub struct CompositeClause {
    pub operator: LogicalOperator,
    pub clauses: *const Clause,
    pub clauses_len: usize,
}

#[derive(Clone)]
#[repr(C)]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Clone)]
#[repr(C)]
pub enum ComparisonOperator {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Clone)]
#[repr(C)]
pub enum Value {
    String(*const c_char),
    Int(i64),
    UInt(u64),
    Bool(bool),
    Bytes(CArray<u8>),
}

#[derive(Clone)]
#[repr(C)]
pub enum Ty {
    Primitive(Primitive),
    Struct(Struct),
    Enum(Enum),
    Tuple(CArray<Ty>),
}

impl From<&dojo_types::schema::Ty> for Ty {
    fn from(value: &dojo_types::schema::Ty) -> Self {
        match value {
            dojo_types::schema::Ty::Primitive(primitive) => {
                let primitive = match primitive {
                    dojo_types::primitive::Primitive::U8(v) => {
                        Primitive::U8((v.clone()).as_mut().unwrap())
                    }
                    dojo_types::primitive::Primitive::U16(v) => {
                        Primitive::U16((v.clone()).as_mut().unwrap())
                    }
                    dojo_types::primitive::Primitive::U32(v) => {
                        Primitive::U32((v.clone()).as_mut().unwrap())
                    }
                    dojo_types::primitive::Primitive::U64(v) => {
                        Primitive::U64((v.clone()).as_mut().unwrap())
                    }
                    dojo_types::primitive::Primitive::U128(v) => {
                        Primitive::U128(v.unwrap().to_be_bytes())
                    }
                    dojo_types::primitive::Primitive::U256(v) => {
                        Primitive::U256(v.unwrap().to_words())
                    }
                    dojo_types::primitive::Primitive::USize(v) => {
                        Primitive::USize((v.clone()).as_mut().unwrap())
                    }
                    dojo_types::primitive::Primitive::Bool(v) => {
                        Primitive::Bool((v.clone()).as_mut().unwrap())
                    }
                    dojo_types::primitive::Primitive::Felt252(v) => {
                        let fe: FieldElement = (&v.unwrap().clone()).into();
                        Primitive::Felt252(&fe)
                    }
                    dojo_types::primitive::Primitive::ClassHash(v) => {
                        let fe: FieldElement = (&v.unwrap().clone()).into();
                        Primitive::Felt252(&fe)
                    }
                    dojo_types::primitive::Primitive::ContractAddress(v) => {
                        let fe: FieldElement = (&v.unwrap().clone()).into();
                        Primitive::Felt252(&fe)
                    }
                };

                Ty::Primitive(primitive)
            }
            dojo_types::schema::Ty::Struct(struct_) => {
                let children = struct_
                    .children
                    .iter()
                    .map(|c| Member {
                        name: CString::new(c.name.clone()).unwrap().into_raw(),
                        ty: (&c.ty.clone()).into(),
                        key: c.key,
                    })
                    .collect::<Vec<_>>();

                Ty::Struct(Struct {
                    name: CString::new(struct_.name.clone()).unwrap().into_raw(),
                    children: CArray {
                        data: children.as_ptr(),
                        data_len: children.len(),
                    },
                })
            }
            dojo_types::schema::Ty::Enum(enum_) => {
                let options = enum_
                    .options
                    .iter()
                    .map(|o| EnumOption {
                        name: CString::new(o.name.clone()).unwrap().into_raw(),
                        ty: (&o.ty.clone()).into(),
                    })
                    .collect::<Vec<_>>();

                Ty::Enum(Enum {
                    name: CString::new(enum_.name.clone()).unwrap().into_raw(),
                    option: enum_.option.unwrap(),
                    options: CArray {
                        data: options.as_ptr(),
                        data_len: options.len(),
                    },
                })
            }
            dojo_types::schema::Ty::Tuple(tuple) => {
                let children = tuple
                    .iter()
                    .map(|c| (&c.clone()).into())
                    .collect::<Vec<_>>();

                Ty::Tuple(CArray {
                    data: children.as_ptr(),
                    data_len: children.len(),
                })
            }
        }
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct Enum {
    pub name: *const c_char,
    pub option: u8,
    pub options: CArray<EnumOption>,
}

#[derive(Clone)]
#[repr(C)]
pub struct EnumOption {
    pub name: *const c_char,
    pub ty: Ty,
}

#[derive(Clone)]
#[repr(C)]
pub struct Struct {
    pub name: *const c_char,
    pub children: CArray<Member>,
}

#[derive(Clone)]
#[repr(C)]
pub struct Member {
    pub name: *const c_char,
    pub ty: Ty,
    pub key: bool,
}

#[derive(Clone)]
#[repr(C)]
pub enum Primitive {
    U8(*const u8),
    U16(*const u16),
    U32(*const u32),
    U64(*const u64),
    // TODO: better way?
    U128([u8; 16]),
    U256([u64; 4]),
    USize(*const u32),
    Bool(*const bool),
    Felt252(*const FieldElement),
    ClassHash(*const FieldElement),
    ContractAddress(*const FieldElement),
}

impl From<&EntityQuery> for dojo_types::schema::EntityQuery {
    fn from(val: &EntityQuery) -> Self {
        dojo_types::schema::EntityQuery {
            model: unsafe { CStr::from_ptr(val.model).to_string_lossy().into_owned() },
            clause: (&val.clause.clone()).into(),
        }
    }
}

impl From<&Clause> for dojo_types::schema::Clause {
    fn from(val: &Clause) -> Self {
        match val {
            Clause::Keys(keys) => dojo_types::schema::Clause::Keys((&keys.clone()).into()),
            Clause::Attribute(attribute) => {
                dojo_types::schema::Clause::Attribute((&attribute.clone()).into())
            }
            Clause::Composite(composite) => {
                dojo_types::schema::Clause::Composite((&composite.clone()).into())
            }
        }
    }
}

impl From<&KeysClause> for dojo_types::schema::KeysClause {
    fn from(val: &KeysClause) -> Self {
        let keys = unsafe { std::slice::from_raw_parts(val.data, val.data_len).to_vec() };

        dojo_types::schema::KeysClause {
            keys: keys.iter().map(|k| k.into()).collect(),
        }
    }
}

impl From<&AttributeClause> for dojo_types::schema::AttributeClause {
    fn from(val: &AttributeClause) -> Self {
        dojo_types::schema::AttributeClause {
            attribute: unsafe { CStr::from_ptr(val.attribute).to_string_lossy().into_owned() },
            operator: (&val.operator.clone()).into(),
            value: (&val.value.clone()).into(),
        }
    }
}

impl From<&CompositeClause> for dojo_types::schema::CompositeClause {
    fn from(val: &CompositeClause) -> Self {
        let operator = &val.operator.clone();
        let clauses = unsafe { std::slice::from_raw_parts(val.clauses, val.clauses_len).to_vec() };

        dojo_types::schema::CompositeClause {
            operator: operator.into(),
            clauses: clauses.iter().map(|c| c.into()).collect(),
        }
    }
}

impl From<&LogicalOperator> for dojo_types::schema::LogicalOperator {
    fn from(val: &LogicalOperator) -> Self {
        match val {
            LogicalOperator::And => dojo_types::schema::LogicalOperator::And,
            LogicalOperator::Or => dojo_types::schema::LogicalOperator::Or,
        }
    }
}

impl From<&ComparisonOperator> for dojo_types::schema::ComparisonOperator {
    fn from(val: &ComparisonOperator) -> Self {
        match val {
            ComparisonOperator::Eq => dojo_types::schema::ComparisonOperator::Eq,
            ComparisonOperator::Neq => dojo_types::schema::ComparisonOperator::Neq,
            ComparisonOperator::Gt => dojo_types::schema::ComparisonOperator::Gt,
            ComparisonOperator::Gte => dojo_types::schema::ComparisonOperator::Gte,
            ComparisonOperator::Lt => dojo_types::schema::ComparisonOperator::Lt,
            ComparisonOperator::Lte => dojo_types::schema::ComparisonOperator::Lte,
        }
    }
}

impl From<&Value> for dojo_types::schema::Value {
    fn from(val: &Value) -> Self {
        match val {
            Value::String(string) => dojo_types::schema::Value::String(unsafe {
                CStr::from_ptr(*string).to_string_lossy().into_owned()
            }),
            Value::Int(int) => dojo_types::schema::Value::Int(*int),
            Value::UInt(uint) => dojo_types::schema::Value::UInt(*uint),
            Value::Bool(bool) => dojo_types::schema::Value::Bool(*bool),
            Value::Bytes(bytes) => unsafe {
                dojo_types::schema::Value::Bytes(
                    std::slice::from_raw_parts(bytes.data, bytes.data_len).to_vec(),
                )
            },
        }
    }
}

pub type EntityKeys = CArray<FieldElement>;
pub type StorageKey = FieldElement;
pub type StorageValue = FieldElement;

#[derive(Clone)]
#[repr(C)]
pub struct CHashMap<K, V> {
    keys: *const K,
    values: *const V,
    len: usize,
}

#[derive(Clone)]
#[repr(C)]
pub struct ModelIndex {
    model: FieldElement,
    keys: CArray<EntityKeys>,
}

#[derive(Clone)]
#[repr(C)]
pub struct ModelStorage {
    metadata: WorldMetadata,
    storage: CHashMap<StorageKey, StorageValue>,
    // a map of model name to a set of entity keys.
    model_index: CHashMap<FieldElement, CArray<EntityKeys>>,
    // listener for storage updates.
    // senders: Mutex<HashMap<u8, Sender<()>>>,
    // listeners: Mutex<HashMap<StorageKey, Vec<u8>>>,
}

// impl From<&torii_client::client::storage::ModelStorage> for ModelStorage {
//     fn from(value: &torii_client::client::storage::ModelStorage) -> Self {
//         let metadata = value.metadata;
//         let storage = value.storage.clone();
//         let model_index = value.model_index.clone();

//         Self {
//             metadata: (&metadata).into(),
//             storage: (&storage).into(),
//             model_index: (&model_index).into(),
//         }
//     }
// }

#[derive(Clone)]
#[repr(C)]
pub struct WorldMetadata {
    pub world_address: FieldElement,
    pub world_class_hash: FieldElement,
    pub executor_address: FieldElement,
    pub executor_class_hash: FieldElement,
    pub models: CHashMap<*const c_char, ModelMetadata>,
}

impl From<&dojo_types::WorldMetadata> for WorldMetadata {
    fn from(value: &dojo_types::WorldMetadata) -> Self {
        WorldMetadata {
            world_address: (&value.world_address.clone()).into(),
            world_class_hash: (&value.world_class_hash.clone()).into(),
            executor_address: (&value.executor_address.clone()).into(),
            executor_class_hash: (&value.executor_class_hash.clone()).into(),
            models: CHashMap {
                keys: value
                    .models
                    .iter()
                    .map(|(k, _)| CString::new(k.clone()).unwrap().into_raw() as *const c_char)
                    .collect::<Vec<_>>()
                    .as_ptr(),
                values: value
                    .models
                    .iter()
                    .map(|(_, v)| (&v.clone()).into())
                    .collect::<Vec<_>>()
                    .as_ptr(),
                len: value.models.len(),
            },
        }
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct ModelMetadata {
    pub schema: Ty,
    pub name: *const c_char,
    pub packed_size: u32,
    pub unpacked_size: u32,
    pub class_hash: FieldElement,
    pub layout: CArray<FieldElement>,
}

impl From<&dojo_types::schema::ModelMetadata> for ModelMetadata {
    fn from(value: &dojo_types::schema::ModelMetadata) -> Self {
        ModelMetadata {
            schema: (&value.schema.clone()).into(),
            name: CString::new(value.name.clone()).unwrap().into_raw(),
            packed_size: value.packed_size.clone(),
            unpacked_size: value.unpacked_size.clone(),
            class_hash: (&value.class_hash.clone()).into(),
            layout: CArray {
                data: value
                    .layout
                    .iter()
                    .map(|v| (&v.clone()).into())
                    .collect::<Vec<_>>()
                    .as_ptr(),
                data_len: value.layout.len(),
            },
        }
    }
}