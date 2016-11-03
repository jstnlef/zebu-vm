use super::common::*;
use std::sync::Arc;

pub struct MuVM {
    // The actual VM
    pub vm: Arc<VM>,

    // Cache C strings. The C client expects `char*` from `name_of`. We assume the client won't
    // call `name_of` very often, so that we don't need to initialise this hashmap on startup.
    name_cache: Mutex<HashMap<MuID, CString>>,
}

/**
 * Implement the methods of MuVM. Most methods implement the C-level methods, and others are
 * rust-level helpers. Most methods are forwarded to the underlying `VM.*` methods.
 */
impl MuVM {
    /**
     * Create a new micro VM instance from scratch.
     */
    pub fn new() -> MuVM {
        MuVM {
            vm: Arc::new(VM::new()),
            // Cache C strings. The C client expects `char*` from `name_of`. We assume the client
            // won't call `name_of` very often, so that we don't need to initialise this hashmap on
            // startup.
            //
            // RwLock won't work because Rust will not let me release the lock after reading
            // because other threads will remove that element from the cache, even though I only
            // monotonically add elements into the `name_cache`. I can't upgrade the lock from read
            // lock to write lock, otherwise it will deadlock.
            name_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn new_context(&self) -> *mut CMuCtx {
        info!("Creating MuCtx...");

        let ctx = MuCtx::new(self);

        let ctx_ptr = Box::into_raw(ctx);

        debug!("The MuCtx address: {:?}", ctx_ptr);

        let cctx = make_new_MuCtx(ctx_ptr as *mut c_void);

        debug!("The C-visible CMuCtx struct address: {:?}", cctx);

        unsafe{ (*ctx_ptr).c_struct = cctx; }

        cctx
    }

    pub fn id_of(&self, name: MuName) -> MuID {
        self.vm.id_of_by_refstring(&name)
    }

    pub fn name_of(&self, id: MuID) -> CMuCString {
        let mut map = self.name_cache.lock().unwrap();

        let cname = map.entry(id).or_insert_with(|| {
            let rustname = self.vm.name_of(id);
            CString::new(rustname).unwrap()
        });

        cname.as_ptr()
    }

    pub fn set_trap_handler(&self, trap_handler: CMuTrapHandler, userdata: CMuCPtr) {
        panic!("Not implemented")
    }

    pub fn compile_to_sharedlib(&self, lib_name: &str, extra_srcs: Vec<String>) {
        extern crate libloading as ll;

        use compiler::*;
        use testutil::aot;

        let compiler = Compiler::new(CompilerPolicy::default(), self.vm.clone());
        let funcs = self.vm.funcs().read().unwrap();
        let mut func_names = vec![];
        // NOTE: this fails because load() API call is not properly implemented yet.
        for (func_id, ref f) in funcs.iter() {
            let func = f.read().unwrap();
            let func_vers = self.vm.func_vers().read().unwrap();
            let mut func_ver = func_vers.get(&func.cur_ver.unwrap()).unwrap().write().unwrap();
            compiler.compile(&mut func_ver);
            func_names.push(func.name().unwrap());
        }
        backend::emit_context(&self.vm);
        aot::link_dylib_with_extra_srcs(func_names, extra_srcs, lib_name);
    }

}

/**
 * Create a micro VM instance, and expose it as a C-visible `*mut CMuVM` pointer.
 *
 * NOTE: When used as an API (such as in tests), please use `mu::vm::api::mu_fastimpl_new` instead.
 *
 * This method is not part of the API defined by the Mu spec. It is used **when the client starts
 * the process and creates the micor VM**. For example, it is used if the client wants to build
 * boot images, or if the client implements most of its parts in C and onlu uses the micro VM as
 * the JIT compiler.
 *
 * The boot image itself should use `VM::resume_vm` to restore the saved the micro VM. There is no
 * need in the boot image itself to expose the `MuVM` structure to the trap handler. Trap handlers
 * only see `MuCtx`, and it is enough for most of the works.
 */
#[no_mangle]
pub extern fn mu_fastimpl_new() -> *mut CMuVM {
    info!("Creating Mu micro VM fast implementation instance...");

    let mvm = Box::new(MuVM::new());
    let mvm_ptr = Box::into_raw(mvm);

    debug!("The MuVM instance address: {:?}", mvm_ptr);

    let c_mvm = make_new_MuVM(mvm_ptr as *mut c_void);

    debug!("The C-visible CMuVM struct address: {:?}", c_mvm);

    c_mvm
}
