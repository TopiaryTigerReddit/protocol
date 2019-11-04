use failure::Fail;
use std::{
    any::{Any, TypeId},
    fmt::{self, Display, Formatter},
};

pub type MethodIndex = u8;

pub struct MethodTypes {
    pub arguments: Vec<TypeId>,
    pub output: TypeId,
}

#[derive(Debug, Fail)]
pub enum CallError {
    Type(u8),
    ArgumentCount(ArgumentCountError),
    OutOfRange(OutOfRangeError),
    IncorrectReceiver(bool),
}

impl Display for CallError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use CallError::{ArgumentCount, IncorrectReceiver, OutOfRange, Type};

        write!(
            f,
            "{}",
            match self {
                Type(position) => format!("invalid type for argument {}", position),
                OutOfRange(error) => format!("{}", error),
                ArgumentCount(error) => format!("{}", error),
                IncorrectReceiver(mutable) => format!(
                    "expected {} receiver, found {}",
                    if *mutable { "mutable" } else { "immutable" },
                    if *mutable { "immutable" } else { "mutable" },
                ),
            }
        )
    }
}

#[derive(Debug, Fail)]
#[fail(display = "method {} out of range", index)]
pub struct OutOfRangeError {
    pub index: MethodIndex,
}

#[derive(Debug, Fail)]
#[fail(display = "got {} arguments, expected {}", got, expected)]
pub struct ArgumentCountError {
    pub expected: usize,
    pub got: usize,
}

#[derive(Debug, Fail)]
#[fail(display = "no method with name {}", name)]
pub struct NameError {
    pub name: String,
}

pub trait Reflected {
    const DO_NOT_IMPLEMENT_THIS_MARKER_TRAIT_MANUALLY: ();
}

#[derive(Debug)]
pub enum Receiver {
    Mutable,
    Immutable,
}

impl Receiver {
    pub fn is_mutable(&self) -> bool {
        use Receiver::Mutable;
        if let Mutable = self {
            true
        } else {
            false
        }
    }
}

pub trait Trait<T: Reflected + ?Sized> {
    fn call(
        &self,
        index: MethodIndex,
        args: Vec<Box<dyn Any + Send>>,
    ) -> Result<Box<dyn Any + Send>, CallError>;
    fn call_mut(
        &mut self,
        index: MethodIndex,
        args: Vec<Box<dyn Any + Send>>,
    ) -> Result<Box<dyn Any + Send>, CallError>;
    fn by_name(&self, name: &'_ str) -> Result<MethodIndex, NameError>;
    fn count(&self) -> MethodIndex;
    fn name_of(&self, index: MethodIndex) -> Result<String, OutOfRangeError>;
    fn types(&self, index: MethodIndex) -> Result<MethodTypes, OutOfRangeError>;
    fn receiver(&self, index: MethodIndex) -> Result<Receiver, OutOfRangeError>;
}
