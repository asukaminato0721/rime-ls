/// consts
use once_cell::sync::Lazy;
use regex::Regex;

pub const NT_PTN: &str = r"((?P<py>[a-zA-Z[:punct:]]+)(?P<se>[0-9]*))$";
pub const AUTO_TRIGGER_PTN: &str = r"[^a-zA-Z[:punct:]\s][a-zA-Z[:punct:]]+[0-9]*$";
// hack "format argument must be a string literal"
macro_rules! trigger_ptn {
    () => {
        r"((?P<tr>[{}])(?P<py>[a-zA-Z[:punct:]]+)(?P<se>[0-9]*))$"
    };
}
pub(crate) use trigger_ptn;

// regex
pub static NT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(NT_PTN).unwrap());
pub static AUTO_TRIGGER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(AUTO_TRIGGER_PTN).unwrap());

// keycodes
pub const K_BACKSPACE: i32 = 0xff08;
