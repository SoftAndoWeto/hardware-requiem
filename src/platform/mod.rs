#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
mod unsupported;

#[cfg(windows)]
pub(crate) use self::windows::collect;

#[cfg(not(windows))]
pub(crate) use self::unsupported::collect;
