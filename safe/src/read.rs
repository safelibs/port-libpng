use crate::read_util::KeepSymbol;
use crate::types::*;
use core::ptr;

unsafe extern "C" {
    fn png_create_read_struct(
        user_png_ver: png_const_charp,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warn_fn: png_error_ptr,
    ) -> png_structp;
    fn png_set_read_fn(png_ptr: png_structrp, io_ptr: png_voidp, read_data_fn: png_rw_ptr);
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

    crate::abi_guard!(png_ptr, {
        unsafe {
            upstream_png_destroy_read_struct(png_ptr_ptr, info_ptr_ptr, end_info_ptr_ptr);
        }
    });
}

#[used]
static FORCE_LINK_READ: [KeepSymbol; 2] = [
    KeepSymbol::new(png_create_read_struct as *mut ()),
    KeepSymbol::new(png_set_read_fn as *mut ()),
];
