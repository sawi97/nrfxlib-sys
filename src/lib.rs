#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

pub mod ctypes {
	pub type c_char = u8;
	pub type c_schar = i8;
	pub type c_uchar = u8;
	pub type c_int = i32;
	pub type c_uint = u32;
	pub type c_long = i32;
	pub type c_ulong = u32;
	pub type c_longlong = i64;
	pub type c_ulonglong = u64;
	pub type c_ushort = u16;
	pub type c_short = i16;

	#[repr(u8)]
	pub enum c_void {
		#[doc(hidden)]
		null = 0,
	}
}

/// Equivalent to a C NULL pointer
pub const NULL: ctypes::c_void = ctypes::c_void::null;

/// Return the error type represented by the return value of [nrf_modem_at_printf] and [nrf_modem_at_cmd].
///
/// * `error` - The return value of [nrf_modem_at_printf] or [nrf_modem_at_cmd].
///
/// # Returns
/// * [NRF_MODEM_AT_ERROR] If the modem response was 'ERROR'.
/// * [NRF_MODEM_AT_CME_ERROR] If the modem response was '+CME ERROR'.
/// * [NRF_MODEM_AT_CMS_ERROR] If the modem response was '+CMS ERROR'.
#[no_mangle]
#[inline(always)]
pub extern "C" fn nrf_modem_at_err_type(error: ctypes::c_int) -> ctypes::c_int {
	(error & 0x00ff0000) >> 16
}

/// Retrieve the specific CME or CMS error from the return value of a [nrf_modem_at_printf] or [nrf_modem_at_cmd] call.
///
///  * `error` - The return value of a [nrf_modem_at_printf] or [nrf_modem_at_cmd] call.
///
/// # Returns
/// The CME or CMS error code.
#[no_mangle]
#[inline(always)]
#[allow(overflowing_literals)]
pub extern "C" fn nrf_modem_at_err(error: ctypes::c_int) -> ctypes::c_int {
	error & 0xff00ffff
}

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
