macro_rules! typedef {
    (($vm: expr) $name: ident = mu_int($len: expr)) => {
        let $name = $vm.declare_type($vm.next_id(), MuType_::int($len));
        $vm.set_name($name.as_entity(), Mu(stringify!($name)));
    };
    (($vm: expr) $name: ident = mu_double) => {
        let $name = $vm.declare_type($vm.next_id(), MuType_::double());
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
    (($vm: expr) $name: ident = mu_uptr($ty: ident)) => {
        let $name = $vm.declare_type($vm.next_id(), MuType_::uptr($ty.clone()));
        $vm.set_name($name.as_entity(), Mu(stringify!($name)));
    };

    (($vm: expr) $name: ident = mu_struct($($ty: ident), *)) => {
        let $name = $vm.declare_type($vm.next_id(), MuType_::mustruct(Mu(stringify!($name)), vec![$($ty.clone()),*]));
        $vm.set_name($name.as_entity(), Mu(stringify!($name)));
    };
    (($vm: expr) $name: ident = mu_struct()) => {
        let $name = $vm.declare_type($vm.next_id(), MuType_::mustruct(Mu(stringify!($name)), vec![]));
        $vm.set_name($name.as_entity(), Mu(stringify!($name)));
    };

    (($vm: expr) $name: ident = mu_hybrid($($ty: ident), *); $var_ty: ident) => {
        let $name = $vm.declare_type($vm.next_id(), MuType_::hybrid(Mu(stringify!($name)), vec![$($ty.clone()), *], $var_ty.clone()));
        $vm.set_name($name.as_entity(), Mu(stringify!($name)));
    };
    (($vm: expr) $name: ident = mu_hybrid(none; $var_ty: ident)) => {
        let $name = $vm.declare_type($vm.next_id(), MuType_::hybrid(Mu(stringify!($name)), vec![], $var_ty.clone()));
        $vm.set_name($name.as_entity(), Mu(stringify!($name)));
    };

    (($vm: expr) $name: ident = mu_funcref($sig: ident)) => {
        let $name = $vm.declare_type($vm.next_id(), MuType_::funcref($sig.clone()));
        $vm.set_name($name.as_entity(), Mu(stringify!($name)));
    }
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

    // GETVARPARTIREF
    (($vm: expr, $fv: ident) $name: ident: $value: ident = GETVARPARTIREF $op: ident (is_ptr: $is_ptr: expr)) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value()]),
            ops:    RwLock::new(vec![$op.clone()]),
            v:      Instruction_::GetVarPartIRef {
                        is_ptr: $is_ptr,
                        base: 0
            }
        });
    };

    // SHIFTIREF
    (($vm: expr, $fv: ident) $name: ident: $value: ident = SHIFTIREF $op: ident $offset: ident (is_ptr: $is_ptr: expr)) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value()]),
            ops:    RwLock::new(vec![$op.clone(), $offset.clone()]),
            v:      Instruction_::ShiftIRef {
                        is_ptr: $is_ptr,
                        base: 0,
                        offset: 1
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

    // CMPOP
    (($vm: expr, $fv: ident) $name: ident: $value: ident = CMPOP ($op: expr) $op1: ident $op2: ident) => {
        let $name = $fv.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed($vm.next_id()),
            value: Some(vec![$value.clone_value()]),
            ops: RwLock::new(vec![$op1.clone(), $op2.clone()]),
            v: Instruction_::CmpOp($op, 0, 1)
        });
    };

    // CONVOP
    (($vm: expr, $fv: ident) $name: ident: $value: ident = CONVOP ($operation: expr) <$ty1: ident $ty2: ident> $operand: ident) => {
        let $name = $fv.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed($vm.next_id()),
            value: Some(vec![$value.clone_value()]),
            ops: RwLock::new(vec![$operand.clone()]),
            v: Instruction_::ConvOp{
                operation: $operation,
                from_ty: $ty1.clone(),
                to_ty: $ty2.clone(),
                operand: 0
            }
        });
    };

    // SELECT
    (($vm: expr, $fv: ident) $name: ident: $value: ident = SELECT $cond: ident $op_true: ident $op_false:ident) => {
        let $name = $fv.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed($vm.next_id()),
            value: Some(vec![$value.clone_value()]),
            ops: RwLock::new(vec![$cond.clone(), $op_true.clone(), $op_false.clone()]),
            v: Instruction_::Select{
                cond: 0,
                true_val: 1,
                false_val: 2
            }
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
                            vec![$($arg.clone()),*].iter().map(|_| {let ret = DestArg::Normal(i); i+=1; ret}).collect()
                        }
            })
        });
    };

    // BRANCH2
    // list all operands first
    // then use vector expr to list operands for each destination
    // (we cannot have two repetition list of different lengths in a macro)
    (($vm: expr, $fv: ident) $name: ident:
        BRANCH2 ($($op: ident), *)
            IF (OP $cond: expr)
            THEN $true_dest : ident ($true_args: expr) WITH $prob: expr,
            ELSE $false_dest: ident ($false_args: expr)
    ) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  None,
            ops:    RwLock::new(vec![$($op.clone()),*]),
            v:      {
                let true_args = {
                    $true_args.iter().map(|x| DestArg::Normal(*x)).collect()
                };

                let false_args = {
                    $false_args.iter().map(|x| DestArg::Normal(*x)).collect()
                };

                Instruction_::Branch2{
                    cond: $cond,
                    true_dest: Destination {
                        target: $true_dest.id(),
                        args: true_args
                    },
                    false_dest: Destination {
                        target: $false_dest.id(),
                        args: false_args
                    },
                    true_prob: $prob
                }
            }
        });
    };

    // CALL
    (($vm: expr, $fv: ident) $name: ident: $res: ident = EXPRCALL ($cc: expr, is_abort: $is_abort: expr) $func: ident ($($val: ident), +)) => {
        let ops = vec![$func.clone(), $($val.clone()), *];
        let ops_len = ops.len();
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$res.clone_value()]),
            ops:    RwLock::new(ops),
            v:      Instruction_::ExprCall {
                        data: CallData {
                            func: 0,
                            args: (1..ops_len).collect(),
                            convention: $cc
                        },
                        is_abort: $is_abort
                    }
        });
    };

    // RET
    (($vm: expr, $fv: ident) $name: ident: RET ($($val: ident), +)) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  None,
            ops:    RwLock::new(vec![$($val.clone()), *]),
            v:      Instruction_::Return({
                        let mut i = 0;
                        vec![$($val.clone()), *].iter().map(|_| {let ret = i; i+= 1; ret}).collect()
                    })
        });
    };
    // RET (no value)
    (($vm: expr, $fv: ident) $name: ident: RET) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  None,
            ops:    RwLock::new(vec![]),
            v:      Instruction_::Return(vec![])
        });
    };
}