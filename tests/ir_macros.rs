macro_rules! typedef {
    (($vm: expr) $name: ident = mu_int($len: expr)) => {
        let $name = $vm.declare_type($vm.next_id(), MuType_::int($len));
        $vm.set_name($name.as_entity(), Mu(stringify!($name)));
    };
    (($vm: expr) $name: ident = mu_ref($ty: ident)) => {
        let $name = $vm.declare_type($vm.next_id(), MuType_::muref($ty.clone()));
        $vm.set_name($name.as_entity(), Mu(stringify!($name)));
    };
    (($vm: expr) $name: ident = mu_iref($ty: ident)) => {
        let $name = $vm.declare_type($vm.next_id(), MuType_::iref($ty.clone()));
        $vm.set_name($name.as_entity(), Mu(stringify!($name)));
    };
    (($vm: expr) $name: ident = mu_struct($($ty: ident), *)) => {
        let $name = $vm.declare_type($vm.next_id(), MuType_::mustruct(Mu(stringify!($name)), vec![$($ty.clone()),*]));
        $vm.set_name($name.as_entity(), Mu(stringify!($name)));
    };
}

macro_rules! constdef {
    (($vm: expr) <$ty: ident> $name: ident = $val: expr) => {
        let $name = $vm.declare_const($vm.next_id(), $ty.clone(), $val);
        $vm.set_name($name.as_entity(), Mu(stringify!($name)));
    }
}

macro_rules! funcsig {
    (($vm: expr) $name: ident = ($($arg_ty: ident),*) -> ($($ret_ty: ident),*)) => {
        let $name = $vm.declare_func_sig($vm.next_id(), vec![$($ret_ty.clone()),*], vec![$($arg_ty.clone()),*]);
        $vm.set_name($name.as_entity(), Mu(stringify!($name)));
    }
}

macro_rules! funcdecl {
    (($vm: expr) <$sig: ident> $name: ident) => {
        let func = MuFunction::new($vm.next_id(), $sig.clone());
        $vm.set_name(func.as_entity(), Mu(stringify!($name)));
        let $name = func.id();
        $vm.declare_func(func);
    }
}

macro_rules! funcdef {
    (($vm: expr) <$sig: ident> $func: ident VERSION $version: ident) => {
        let mut $version = MuFunctionVersion::new($vm.next_id(), $func, $sig.clone());
        $vm.set_name($version.as_entity(), Mu(stringify!($version)));
    }
}

macro_rules! define_func_ver {
    (($vm: expr) $fv: ident (entry: $entry: ident){$($blk: ident), *}) => {
        $fv.define(FunctionContent{
            entry: $entry.id(),
            blocks: hashmap!{
                $($blk.id() => $blk),*
            }
        });

        $vm.define_func_version($fv);
    }
}

macro_rules! block {
    (($vm: expr, $fv: ident) $name: ident) => {
        let mut $name = Block::new($vm.next_id());
        $vm.set_name($name.as_entity(), Mu(stringify!($name)));
    }
}

macro_rules! define_block {
    (($vm: expr, $fv: ident) $name: ident ($($arg: ident), *) {$($inst: ident), *}) => {
        $name.content = Some(BlockContent{
            args: vec![$($arg.clone_value()), *],
            exn_arg: None,
            body: vec![$($inst), *],
            keepalives: None
        });
    }
}

macro_rules! ssa {
    (($vm: expr, $fv: ident) <$ty: ident> $name: ident) => {
        let $name = $fv.new_ssa($vm.next_id(), $ty.clone());
        $vm.set_name($name.as_entity(), Mu(stringify!($name)));
    }
}

macro_rules! consta {
    (($vm: expr, $fv: ident) $name: ident = $c: ident) => {
        let $name = $fv.new_constant($c.clone());
    }
}

macro_rules! inst {
    // NEW
    (($vm: expr, $fv: ident) $name: ident: $value: ident = NEW <$ty: ident>) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value()]),
            ops:    RwLock::new(vec![]),
            v:      Instruction_::New($ty.clone())
        });
    };

    // GETIREF
    (($vm: expr, $fv: ident) $name: ident: $value: ident = GETIREF $op: ident) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value()]),
            ops:    RwLock::new(vec![$op.clone()]),
            v:      Instruction_::GetIRef(0)
        });
    };

    // GETFIELDIREF
    (($vm: expr, $fv: ident) $name: ident: $value: ident = GETFIELDIREF $op: ident (is_ptr: $is_ptr: expr, index: $index: expr)) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value()]),
            ops:    RwLock::new(vec![$op.clone()]),
            v:      Instruction_::GetFieldIRef {
                        is_ptr: $is_ptr,
                        base: 0,
                        index: $index
            }
        });
    };

    // STORE
    (($vm: expr, $fv: ident) $name: ident: STORE $loc: ident $val: ident (is_ptr: $is_ptr: expr, order: $order: expr)) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  None,
            ops:    RwLock::new(vec![$loc.clone(), $val.clone()]),
            v:      Instruction_::Store {
                        is_ptr: $is_ptr,
                        order: $order,
                        mem_loc: 0,
                        value: 1
            }
        });
    };

    // LOAD
    (($vm: expr, $fv: ident) $name: ident: $value: ident = LOAD $loc: ident (is_ptr: $is_ptr: expr, order: $order: expr)) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value()]),
            ops:    RwLock::new(vec![$loc.clone()]),
            v:      Instruction_::Load {
                        is_ptr: $is_ptr,
                        order: $order,
                        mem_loc: 0
            }
        });
    };

    // BINOP
    (($vm: expr, $fv: ident) $name: ident: $value: ident = BINOP ($op: expr) $op1: ident $op2: ident) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value()]),
            ops:    RwLock::new(vec![$op1.clone(), $op2.clone()]),
            v:      Instruction_::BinOp($op, 0, 1)
        });
    };

    // BRANCH
    (($vm: expr, $fv: ident) $name: ident: BRANCH $dest: ident ($($arg: ident), *)) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  None,
            ops:    RwLock::new(vec![$($arg.clone()),*]),
            v:      Instruction_::Branch1(Destination{
                        target: $dest.id(),
                        args: {
                            let mut i =0;
                            vec![$($arg.clone()),*].iter().map(|x| {let ret = DestArg::Normal(i); i+=1; ret}).collect()
                        }
            })
        });
    };

    // RET
    (($vm: expr, $fv: ident) $name: ident: RET ($($val: ident), *)) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  None,
            ops:    RwLock::new(vec![$($val.clone()), *]),
            v:      Instruction_::Return({
                        let mut i = 0;
                        vec![$($val.clone()), *].iter().map(|x| {let ret = i; i+= 1; ret}).collect()
                    })
        });
    };
}