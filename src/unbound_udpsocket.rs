use cvt;
use libc;
use std::os::fd::FromRawFd;

use crate::util::IpAddrKind;
use crate::util::IpAddrKind::{V4,V6};

#[cfg(any(
    target_os = "android",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "hurd",
    target_os = "linux",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "nto",
))]
fn raw_socket_fd(fam: libc::c_int, ty: libc::c_int) -> std::io::Result<libc::c_int> {
    unsafe{
        // On platforms that support it we pass the SOCK_CLOEXEC
        // flag to atomically create the socket and set it as
        // CLOEXEC. On Linux this was added in 2.6.27.
        let fd:libc::c_int = cvt::cvt(libc::socket(fam, ty | libc::SOCK_CLOEXEC, 0))?;
        Ok(fd)
    }
}

#[cfg(all(not(any(
    target_os = "android",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "hurd",
    target_os = "linux",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "nto",
)),not(target_vendor = "apple"))
)]
fn raw_socket_fd(fam: libc::c_int, ty: libc::c_int) -> std::io::Result<libc::c_int> {
    unsafe{
        let fd:libc::c_int = cvt::cvt(libc::socket(fam, ty, 0))?;
        cvt::cvt(libc::ioctl(fd, libc::FIOCLEX))?;
        Ok(fd)
    }
}

#[cfg(all(not(any(
    target_os = "android",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "hurd",
    target_os = "linux",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "nto",
)),target_vendor = "apple")
)]
fn raw_socket_fd(fam: libc::c_int, ty: libc::c_int) -> std::io::Result<libc::c_int> {
    unsafe{
        let fd:libc::c_int = cvt::cvt(libc::socket(fam, ty, 0))?;
        cvt::cvt(libc::ioctl(fd, libc::FIOCLEX))?;
        // macOS and iOS use `SO_NOSIGPIPE` as a `setsockopt`
        // flag to disable `SIGPIPE` emission on socket.
        let opval: libc::c_int=1;
        cvt::cvt(libc::setsockopt(fd, libc::SOL_SOCKET,  libc::SO_NOSIGPIPE,&opval as *const libc::c_int as *const _,std::mem::size_of::<libc::c_int>() as libc::socklen_t,))?;
        Ok(fd)
    }
}

pub fn unbound_udpsocket(kind:IpAddrKind) -> std::io::Result<std::net::UdpSocket>{
    unsafe{
    let fd=raw_socket_fd(
        match kind{
            V4=>{
                libc::AF_INET
            }
            V6=>{
                libc::AF_INET6
            }
        },
        libc::SOCK_DGRAM)?;
        Ok(std::net::UdpSocket::from_raw_fd(fd))
    }
}