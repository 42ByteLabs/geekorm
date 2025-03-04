use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use std::{
    any::{Any, TypeId},
    fmt::Debug,
};
use syn::{GenericArgument, Ident, Type, TypePath};

#[cfg(feature = "chrono")]
use chrono::DateTime;
#[cfg(feature = "uuid")]
use uuid::Uuid;

mod column;
mod columntypes;
mod table;

use crate::{
    attr::{GeekAttribute, GeekAttributeKeys, GeekAttributeValue},
    internal::TableState,
};
pub(crate) use column::{ColumnDerive, ColumnsDerive};
pub(crate) use columntypes::{ColumnTypeDerive, ColumnTypeOptionsDerive};
pub(crate) use table::TableDerive;
