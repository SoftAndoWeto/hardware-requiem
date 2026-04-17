#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
mod unsupported;

#[cfg(windows)]
pub(crate) use self::windows::collect;

#[cfg(not(windows))]
pub(crate) use self::unsupported::collect;

#[cfg(windows)]
pub(crate) use self::windows::collect_os_info;

#[cfg(not(windows))]
pub(crate) use self::unsupported::collect_os_info;
