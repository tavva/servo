/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font::UsedFontStyle;
use platform::font::FontHandle;
use font_context::FontContextHandleMethods;
use platform::font_list::path_from_identifier;

use freetype::freetype::FTErrorMethods;
use freetype::freetype::FT_Add_Default_Modules;
use freetype::freetype::FT_Done_FreeType;
use freetype::freetype::FT_Library;
use freetype::freetype::FT_Memory;
use freetype::freetype::FT_New_Library;
use freetype::freetype::struct_FT_MemoryRec_;

use std::ptr;
use std::rc::Rc;

use libc;
use libc::{c_void, c_long, size_t, malloc};
use std::mem;

extern fn ft_alloc(_mem: FT_Memory, size: c_long) -> *c_void {
    unsafe {
        let ptr = libc::malloc(size as size_t);
        ptr as *c_void
    }
}

extern fn ft_free(_mem: FT_Memory, block: *c_void) {
    unsafe {
        libc::free(block as *mut c_void);
    }
}

extern fn ft_realloc(_mem: FT_Memory, _cur_size: c_long, new_size: c_long, block: *c_void) -> *c_void {
    unsafe {
        let ptr = libc::realloc(block as *mut c_void, new_size as size_t);
        ptr as *c_void
    }
}

#[deriving(Clone)]
pub struct FreeTypeLibraryHandle {
    pub ctx: FT_Library,
}

#[deriving(Clone)]
pub struct FontContextHandle {
    pub ctx: Rc<FreeTypeLibraryHandle>,
}

impl Drop for FreeTypeLibraryHandle {
    fn drop(&mut self) {
        assert!(self.ctx.is_not_null());
        unsafe { FT_Done_FreeType(self.ctx) };
    }
}

impl FontContextHandle {
    pub fn new() -> FontContextHandle {
        unsafe {

            let ptr = libc::malloc(mem::size_of::<struct_FT_MemoryRec_>() as size_t);
            let allocator: &mut struct_FT_MemoryRec_ = mem::transmute(ptr);
            mem::overwrite(allocator, struct_FT_MemoryRec_ {
                user: ptr::null(),
                alloc: ft_alloc,
                free: ft_free,
                realloc: ft_realloc,
            });

            let ctx: FT_Library = ptr::null();

            let result = FT_New_Library(ptr as FT_Memory, &ctx);
            if !result.succeeded() { fail!("Unable to initialize FreeType library"); }

            FT_Add_Default_Modules(ctx);

            FontContextHandle {
                ctx: Rc::new(FreeTypeLibraryHandle { ctx: ctx }),
            }
        }
    }
}

impl FontContextHandleMethods for FontContextHandle {
    fn create_font_from_identifier(&self, name: String, style: UsedFontStyle)
                                -> Result<FontHandle, ()> {
        debug!("Creating font handle for {:s}", name);
        path_from_identifier(name, &style).and_then(|file_name| {
            debug!("Opening font face {:s}", file_name);
            FontHandle::new_from_file(self, file_name.as_slice(), &style)
        })
    }
}

