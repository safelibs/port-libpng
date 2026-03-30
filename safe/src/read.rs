use crate::chunks::clear_read_state;
use crate::read_util::KeepSymbol;
use crate::types::*;
use core::ptr;

unsafe extern "C" {
    fn png_create_read_struct();
    fn png_set_read_fn();
    fn upstream_png_destroy_read_struct(
        png_ptr_ptr: png_structpp,
        info_ptr_ptr: png_infopp,
        end_info_ptr_ptr: png_infopp,
    );
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_destroy_read_struct(
    png_ptr_ptr: png_structpp,
    info_ptr_ptr: png_infopp,
    end_info_ptr_ptr: png_infopp,
) {
    let png_ptr = if png_ptr_ptr.is_null() {
        ptr::null_mut()
    } else {
        unsafe { *png_ptr_ptr }
    };
    clear_read_state(png_ptr);
    unsafe {
        upstream_png_destroy_read_struct(png_ptr_ptr, info_ptr_ptr, end_info_ptr_ptr);
    }
}

#[used]
static FORCE_LINK_READ: [KeepSymbol; 2] = [
    KeepSymbol(png_create_read_struct as *const ()),
    KeepSymbol(png_set_read_fn as *const ()),
];
