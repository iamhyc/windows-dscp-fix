use pyo3::prelude::*;
use pyo3::exceptions::PyOSError;
// Windows Libraries: WinSock2, QOS2
use std::net::Ipv4Addr;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Networking::WinSock::SOCKET;
use windows::Win32::Networking::WinSock::{AF_INET,SOCKADDR_IN,WSAConnect};
use windows::Win32::Foundation::GetLastError;
use windows::Win32::NetworkManagement::QoS::{QOS_VERSION, QOSCreateHandle, QOSAddSocketToFlow, QOSSetFlow};
use windows::Win32::NetworkManagement::QoS::{QOS_SET_FLOW, QOSTrafficTypeBestEffort, QOS_NON_ADAPTIVE_FLOW, QOSSetOutgoingDSCPValue};
// Windows Libraries: Debug
use windows::core::PWSTR;
use windows::Win32::System::Diagnostics::Debug::FormatMessageW;
use windows::Win32::System::Diagnostics::Debug::FORMAT_MESSAGE_ALLOCATE_BUFFER;
use windows::Win32::System::Diagnostics::Debug::{FORMAT_MESSAGE_FROM_SYSTEM,FORMAT_MESSAGE_IGNORE_INSERTS};

type WinError = (u32, String);

#[cfg(windows)]
unsafe fn extract_error_message(prefix:&str) -> Option<WinError> {
    let dw_flags = FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS;
    let dw_message_id = GetLastError().0;
    let mut lp_buffer = PWSTR::null();

    FormatMessageW(dw_flags, None, dw_message_id, 0, PWSTR(&mut lp_buffer.0 as *mut _ as *mut _), 0, None);
    assert!( !lp_buffer.is_null() );
    // eprintln!("{}, {}: {}", prefix, dw_message_id, lp_buffer.to_string().unwrap());
    let dw_message = lp_buffer.to_string().ok()?;
    let dw_message = format!("{}: {}", prefix, dw_message.trim());
    Some( (dw_message_id, dw_message) )
}

#[cfg(windows)]
unsafe fn _connect_socket(raw_sock:SOCKET) {
    let local_addr = "127.0.0.0".parse::<Ipv4Addr>().unwrap().into();
    let local_sockaddr = SOCKADDR_IN{ sin_family:AF_INET, sin_port:0, sin_addr:local_addr, ..Default::default() };
    WSAConnect(raw_sock, &local_sockaddr as *const _ as *const _, std::mem::size_of::<SOCKADDR_IN>() as i32, None, None, None, None);
}

#[cfg(windows)]
fn set_socket_dscp(fd:usize, dscp:u8) -> Option<WinError> {
    let raw_sock = SOCKET(fd);
    let mut flow_id = 0;
    let mut qos_handle = HANDLE(0);
    let value_size = std::mem::size_of::<u32>();
    let qos_version = QOS_VERSION{ MajorVersion:1, MinorVersion:0 };

    unsafe {
        _connect_socket(raw_sock);

        if !QOSCreateHandle(&qos_version as *const _, &mut qos_handle as *mut HANDLE).as_bool() {
            return extract_error_message("QOSCreateHandle");
        }

        if !QOSAddSocketToFlow(qos_handle, raw_sock, None, QOSTrafficTypeBestEffort, QOS_NON_ADAPTIVE_FLOW, &mut flow_id as *mut _).as_bool() {
            return extract_error_message("QOSAddSocketToFlow");
        }

        if !QOSSetFlow(qos_handle,flow_id,QOSSetOutgoingDSCPValue as QOS_SET_FLOW,value_size as u32,&dscp as *const _ as *const _,0,None).as_bool() {
            return extract_error_message("QOSSetFlow");
        }
    }

    None
}

fn raise_os_error(res:Option<WinError>) -> PyResult<()> {
    match res {
        None => Ok(()),
        Some((err_code, err_msg)) => {
            let msg = format!("[WinError {}] {}", err_code, err_msg);
            Err( PyOSError::new_err(msg) )
        }
    }
}

/// Set DSCP value from ToS value
#[pyfunction]
#[pyo3(name = "set_socket_tos")]
fn py_set_socket_tos(fd:usize, tos: u8) -> PyResult<()> {
    let dscp = tos >> 2; //DSCP value is the high-order 6 bits of the ToS
    raise_os_error( set_socket_dscp(fd, dscp) )
}

/// Set DSCP value
#[pyfunction]
#[pyo3(name = "set_socket_dscp")]
fn py_set_socket_dscp(fd:usize, dscp: u8) -> PyResult<()> {
    raise_os_error( set_socket_dscp(fd, dscp) )
}

/// A Python module implemented in Rust.
#[pymodule]
fn windows_dscp_fix(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_set_socket_tos, m)?)?;
    m.add_function(wrap_pyfunction!(py_set_socket_dscp, m)?)?;
    Ok(())
}
