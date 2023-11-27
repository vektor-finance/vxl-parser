use nom_locate::LocatedSpan;
use nom_tracable::TracableInfo;

use crate::{Error, ErrorKind, Node, Tree};

pub type Span<'a> = LocatedSpan<&'a str, TracableInfo>;

pub type SResult<O, E> = std::result::Result<O, E>;
pub type Result<'a, I = Span<'a>, O = Node, E = (I, ErrorKind)> = SResult<(I, O), nom::Err<E>>;
pub type OResult<'a> = SResult<Tree, Box<dyn Error + 'a>>;
