use crate::types::*;
use core::ffi::c_int;
use libc::FILE;

unsafe extern "C" {
    pub(crate) fn bridge_png_calloc(
        png_ptr: png_const_structrp,
        size: png_alloc_size_t,
    ) -> png_voidp;
    pub(crate) fn bridge_png_malloc(
        png_ptr: png_const_structrp,
        size: png_alloc_size_t,
    ) -> png_voidp;
    pub(crate) fn bridge_png_malloc_default(
        png_ptr: png_const_structrp,
        size: png_alloc_size_t,
    ) -> png_voidp;
    pub(crate) fn bridge_png_malloc_warn(
        png_ptr: png_const_structrp,
        size: png_alloc_size_t,
    ) -> png_voidp;
    pub(crate) fn bridge_png_free(png_ptr: png_const_structrp, ptr_to_free: png_voidp);
    pub(crate) fn bridge_png_free_default(
        png_ptr: png_const_structrp,
        ptr_to_free: png_voidp,
    );
    pub(crate) fn bridge_png_set_mem_fn(
        png_ptr: png_structrp,
        mem_ptr: png_voidp,
        malloc_fn: png_malloc_ptr,
        free_fn: png_free_ptr,
    );
    pub(crate) fn bridge_png_get_mem_ptr(png_ptr: png_const_structrp) -> png_voidp;
    pub(crate) fn bridge_png_create_read_struct(
        user_png_ver: png_const_charp,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warn_fn: png_error_ptr,
    ) -> png_structp;
    pub(crate) fn bridge_png_create_read_struct_2(
        user_png_ver: png_const_charp,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warn_fn: png_error_ptr,
        mem_ptr: png_voidp,
        malloc_fn: png_malloc_ptr,
        free_fn: png_free_ptr,
    ) -> png_structp;
    pub(crate) fn bridge_png_create_write_struct(
        user_png_ver: png_const_charp,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warn_fn: png_error_ptr,
    ) -> png_structp;
    pub(crate) fn bridge_png_create_write_struct_2(
        user_png_ver: png_const_charp,
        error_ptr: png_voidp,
        error_fn: png_error_ptr,
        warn_fn: png_error_ptr,
        mem_ptr: png_voidp,
        malloc_fn: png_malloc_ptr,
        free_fn: png_free_ptr,
    ) -> png_structp;
    pub(crate) fn bridge_png_create_info_struct(png_ptr: png_const_structrp) -> png_infop;
    pub(crate) fn bridge_png_destroy_info_struct(
        png_ptr: png_const_structrp,
        info_ptr_ptr: png_infopp,
    );
    pub(crate) fn bridge_png_info_init_3(ptr_ptr: png_infopp, png_info_struct_size: usize);
    pub(crate) fn bridge_png_data_freer(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        freer: c_int,
        mask: png_uint_32,
    );
    pub(crate) fn bridge_png_free_data(
        png_ptr: png_const_structrp,
        info_ptr: png_inforp,
        mask: png_uint_32,
        num: c_int,
    );
    pub(crate) fn bridge_png_destroy_read_struct(
        png_ptr_ptr: png_structpp,
        info_ptr_ptr: png_infopp,
        end_info_ptr_ptr: png_infopp,
    );
    pub(crate) fn bridge_png_destroy_write_struct(
        png_ptr_ptr: png_structpp,
        info_ptr_ptr: png_infopp,
    );

    pub(crate) fn bridge_png_write_info_before_PLTE(
        png_ptr: png_structrp,
        info_ptr: png_const_inforp,
    );
    pub(crate) fn bridge_png_write_info(png_ptr: png_structrp, info_ptr: png_const_inforp);
    pub(crate) fn bridge_png_write_row(png_ptr: png_structrp, row: png_const_bytep);
    pub(crate) fn bridge_png_write_rows(
        png_ptr: png_structrp,
        row: png_bytepp,
        num_rows: png_uint_32,
    );
    pub(crate) fn bridge_png_write_image(png_ptr: png_structrp, image: png_bytepp);
    pub(crate) fn bridge_png_write_end(png_ptr: png_structrp, info_ptr: png_inforp);
    pub(crate) fn bridge_png_write_png(
        png_ptr: png_structrp,
        info_ptr: png_inforp,
        transforms: png_uint_32,
        params: png_voidp,
    );
    pub(crate) fn bridge_png_set_flush(png_ptr: png_structrp, nrows: c_int);
    pub(crate) fn bridge_png_write_flush(png_ptr: png_structrp);

    pub(crate) fn bridge_png_get_compression_buffer_size(
        png_ptr: png_const_structrp,
    ) -> usize;
    pub(crate) fn bridge_png_write_sig(png_ptr: png_structrp);
    pub(crate) fn bridge_png_write_chunk(
        png_ptr: png_structrp,
        chunk_name: png_const_bytep,
        data: png_const_bytep,
        length: usize,
    );
    pub(crate) fn bridge_png_write_chunk_start(
        png_ptr: png_structrp,
        chunk_name: png_const_bytep,
        length: png_uint_32,
    );
    pub(crate) fn bridge_png_write_chunk_data(
        png_ptr: png_structrp,
        data: png_const_bytep,
        length: usize,
    );
    pub(crate) fn bridge_png_write_chunk_end(png_ptr: png_structrp);
    pub(crate) fn bridge_png_set_compression_buffer_size(
        png_ptr: png_structrp,
        size: usize,
    );
    pub(crate) fn bridge_png_set_compression_level(png_ptr: png_structrp, level: c_int);
    pub(crate) fn bridge_png_set_compression_mem_level(
        png_ptr: png_structrp,
        mem_level: c_int,
    );
    pub(crate) fn bridge_png_set_compression_method(png_ptr: png_structrp, method: c_int);
    pub(crate) fn bridge_png_set_compression_strategy(
        png_ptr: png_structrp,
        strategy: c_int,
    );
    pub(crate) fn bridge_png_set_compression_window_bits(
        png_ptr: png_structrp,
        window_bits: c_int,
    );
    pub(crate) fn bridge_png_set_text_compression_level(png_ptr: png_structrp, level: c_int);
    pub(crate) fn bridge_png_set_text_compression_mem_level(
        png_ptr: png_structrp,
        mem_level: c_int,
    );
    pub(crate) fn bridge_png_set_text_compression_method(
        png_ptr: png_structrp,
        method: c_int,
    );
    pub(crate) fn bridge_png_set_text_compression_strategy(
        png_ptr: png_structrp,
        strategy: c_int,
    );
    pub(crate) fn bridge_png_set_text_compression_window_bits(
        png_ptr: png_structrp,
        window_bits: c_int,
    );

    pub(crate) fn bridge_png_set_filter(
        png_ptr: png_structrp,
        method: c_int,
        filters: c_int,
    );
    pub(crate) fn bridge_png_set_filter_heuristics(
        png_ptr: png_structrp,
        heuristic_method: c_int,
        num_weights: c_int,
        filter_weights: png_const_doublep,
        filter_costs: png_const_doublep,
    );
    pub(crate) fn bridge_png_set_filter_heuristics_fixed(
        png_ptr: png_structrp,
        heuristic_method: c_int,
        num_weights: c_int,
        filter_weights: png_const_fixed_point_p,
        filter_costs: png_const_fixed_point_p,
    );
    pub(crate) fn bridge_png_set_add_alpha(
        png_ptr: png_structrp,
        filler: png_uint_32,
        flags: c_int,
    );
    pub(crate) fn bridge_png_set_filler(
        png_ptr: png_structrp,
        filler: png_uint_32,
        flags: c_int,
    );
    pub(crate) fn bridge_png_set_packing(png_ptr: png_structrp);
    pub(crate) fn bridge_png_set_packswap(png_ptr: png_structrp);
    pub(crate) fn bridge_png_set_swap(png_ptr: png_structrp);

    pub(crate) fn bridge_png_image_begin_read_from_file(
        image: png_imagep,
        file_name: png_const_charp,
    ) -> c_int;
    pub(crate) fn bridge_png_image_begin_read_from_stdio(
        image: png_imagep,
        file: *mut FILE,
    ) -> c_int;
    pub(crate) fn bridge_png_image_begin_read_from_memory(
        image: png_imagep,
        memory: png_const_voidp,
        size: usize,
    ) -> c_int;
    pub(crate) fn bridge_png_image_finish_read(
        image: png_imagep,
        background: png_const_colorp,
        buffer: png_voidp,
        row_stride: png_int_32,
        colormap: png_voidp,
    ) -> c_int;
    pub(crate) fn bridge_png_image_write_to_file(
        image: png_imagep,
        file_name: png_const_charp,
        convert_to_8bit: c_int,
        buffer: png_const_voidp,
        row_stride: png_int_32,
        colormap: png_const_voidp,
    ) -> c_int;
    pub(crate) fn bridge_png_image_write_to_stdio(
        image: png_imagep,
        file: *mut FILE,
        convert_to_8bit: c_int,
        buffer: png_const_voidp,
        row_stride: png_int_32,
        colormap: png_const_voidp,
    ) -> c_int;
    pub(crate) fn bridge_png_image_write_to_memory(
        image: png_imagep,
        memory: png_voidp,
        memory_bytes: *mut png_alloc_size_t,
        convert_to_8bit: c_int,
        buffer: png_const_voidp,
        row_stride: png_int_32,
        colormap: png_const_voidp,
    ) -> c_int;
    pub(crate) fn bridge_png_image_free(image: png_imagep);
}

pub(crate) use bridge_png_calloc as alloc_zeroed;
pub(crate) use bridge_png_create_info_struct as create_info_handle;
pub(crate) use bridge_png_create_read_struct as create_read_handle;
pub(crate) use bridge_png_create_read_struct_2 as create_read_handle_with_hooks;
pub(crate) use bridge_png_create_write_struct as create_write_handle;
pub(crate) use bridge_png_create_write_struct_2 as create_write_handle_with_hooks;
pub(crate) use bridge_png_data_freer as mark_data_freer;
pub(crate) use bridge_png_destroy_info_struct as destroy_info_handle;
pub(crate) use bridge_png_destroy_read_struct as destroy_read_handle;
pub(crate) use bridge_png_destroy_write_struct as destroy_write_handle;
pub(crate) use bridge_png_free as release;
pub(crate) use bridge_png_free_data as release_data;
pub(crate) use bridge_png_free_default as release_default;
pub(crate) use bridge_png_get_compression_buffer_size as compression_buffer_size;
pub(crate) use bridge_png_get_mem_ptr as memory_ptr;
pub(crate) use bridge_png_image_begin_read_from_file as image_begin_read_from_file;
pub(crate) use bridge_png_image_begin_read_from_memory as image_begin_read_from_memory;
pub(crate) use bridge_png_image_begin_read_from_stdio as image_begin_read_from_stdio;
pub(crate) use bridge_png_image_finish_read as image_finish_read;
pub(crate) use bridge_png_image_free as image_free;
pub(crate) use bridge_png_image_write_to_file as image_write_to_file;
pub(crate) use bridge_png_image_write_to_memory as image_write_to_memory;
pub(crate) use bridge_png_image_write_to_stdio as image_write_to_stdio;
pub(crate) use bridge_png_info_init_3 as init_info_handle;
pub(crate) use bridge_png_malloc as alloc;
pub(crate) use bridge_png_malloc_default as alloc_default;
pub(crate) use bridge_png_malloc_warn as alloc_warn;
pub(crate) use bridge_png_set_add_alpha as set_add_alpha;
pub(crate) use bridge_png_set_compression_buffer_size as set_compression_buffer_size;
pub(crate) use bridge_png_set_compression_level as set_compression_level;
pub(crate) use bridge_png_set_compression_mem_level as set_compression_mem_level;
pub(crate) use bridge_png_set_compression_method as set_compression_method;
pub(crate) use bridge_png_set_compression_strategy as set_compression_strategy;
pub(crate) use bridge_png_set_compression_window_bits as set_compression_window_bits;
pub(crate) use bridge_png_set_filter as set_filter;
pub(crate) use bridge_png_set_filter_heuristics as set_filter_heuristics;
pub(crate) use bridge_png_set_filter_heuristics_fixed as set_filter_heuristics_fixed;
pub(crate) use bridge_png_set_filler as set_filler;
pub(crate) use bridge_png_set_flush as set_flush_rows;
pub(crate) use bridge_png_set_mem_fn as set_memory_hooks;
pub(crate) use bridge_png_set_packing as set_packing;
pub(crate) use bridge_png_set_packswap as set_packswap;
pub(crate) use bridge_png_set_swap as set_swap;
pub(crate) use bridge_png_set_text_compression_level as set_text_compression_level;
pub(crate) use bridge_png_set_text_compression_mem_level as set_text_compression_mem_level;
pub(crate) use bridge_png_set_text_compression_method as set_text_compression_method;
pub(crate) use bridge_png_set_text_compression_strategy as set_text_compression_strategy;
pub(crate) use bridge_png_set_text_compression_window_bits as set_text_compression_window_bits;
pub(crate) use bridge_png_write_chunk as write_chunk;
pub(crate) use bridge_png_write_chunk_data as write_chunk_data;
pub(crate) use bridge_png_write_chunk_end as finish_chunk;
pub(crate) use bridge_png_write_chunk_start as start_chunk;
pub(crate) use bridge_png_write_end as write_end;
pub(crate) use bridge_png_write_flush as flush_output;
pub(crate) use bridge_png_write_image as write_image;
pub(crate) use bridge_png_write_info as write_info;
pub(crate) use bridge_png_write_info_before_PLTE as write_info_before_palette;
pub(crate) use bridge_png_write_png as write_png;
pub(crate) use bridge_png_write_row as write_row;
pub(crate) use bridge_png_write_rows as write_rows;
pub(crate) use bridge_png_write_sig as write_signature;
