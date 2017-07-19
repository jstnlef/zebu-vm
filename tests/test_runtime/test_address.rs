use utils::Address;
//use std::sync::atomic::Ordering;

#[test]
fn test_align_up() {
    let addr = unsafe { Address::from_usize(0) };
    let aligned = addr.align_up(8);

    assert_eq!(aligned, addr);

    let addr = unsafe { Address::from_usize(1) };
    let aligned = addr.align_up(8);

    assert_eq!(aligned, unsafe { Address::from_usize(8) });
}

#[test]
fn test_is_aligned() {
    let addr = unsafe { Address::from_usize(0) };
    assert!(addr.is_aligned_to(8));

    let addr = unsafe { Address::from_usize(1) };
    assert!(!addr.is_aligned_to(8));

    let addr = unsafe { Address::from_usize(8) };
    assert!(addr.is_aligned_to(8));
}

//#[test]
//fn test_load_order_u64() {
//    let mem = Box::new(42u64);
//    let ptr = Box::into_raw(mem);
//    let addr = Address::from_mut_ptr(ptr);
//
//    unsafe {
//        let value_relaxed : u64 = addr.load_order(Ordering::Relaxed);
//        assert_eq!(value_relaxed, 42);
//
//        let value_seqcst : u64 = addr.load_order(Ordering::SeqCst);
//        assert_eq!(value_seqcst, 42);
//
//        let value_acquire : u64 = addr.load_order(Ordering::Acquire);
//        assert_eq!(value_acquire, 42);
//    }
//}
//
//#[test]
//fn test_store_order_u64() {
//    let mem = Box::new(0u64);
//    let ptr = Box::into_raw(mem);
//    let addr = Address::from_mut_ptr(ptr);
//
//    unsafe {
//        let expect : u64 = 42;
//        addr.store_order(expect, Ordering::Relaxed);
//        let val : u64 = addr.load();
//        assert_eq!(val, expect);
//
//        let expect : u64 = 21;
//        addr.store_order(expect, Ordering::Release);
//        let val : u64 = addr.load();
//        assert_eq!(val, expect);
//
//        let expect : u64 = 10;
//        addr.store_order(expect, Ordering::SeqCst);
//        let val : u64 = addr.load();
//        assert_eq!(val, expect);
//    }
//}
//
//#[test]
//fn test_load_order_u32() {
//    let mem = Box::new(-1isize);
//    let ptr = Box::into_raw(mem);
//    let addr = Address::from_mut_ptr(ptr);
//
//    unsafe {
//        addr.store(42u32);
//
//        let value_relaxed : u32 = addr.load_order(Ordering::Relaxed);
//        assert_eq!(value_relaxed, 42);
//
//        let value_seqcst : u32 = addr.load_order(Ordering::SeqCst);
//        assert_eq!(value_seqcst, 42);
//
//        let value_acquire : u32 = addr.load_order(Ordering::Acquire);
//        assert_eq!(value_acquire, 42);
//    }
//}
//
//#[test]
//fn test_load_order_f64() {
//    let mem = Box::new(42.0f64);
//    let ptr = Box::into_raw(mem);
//    let addr = Address::from_mut_ptr(ptr);
//
//    unsafe {
//        let value_relaxed : f64 = addr.load_order(Ordering::Relaxed);
//        assert_eq!(value_relaxed, 42f64);
//
//        let value_seqcst : f64 = addr.load_order(Ordering::SeqCst);
//        assert_eq!(value_seqcst, 42f64);
//
//        let value_acquire : f64 = addr.load_order(Ordering::Acquire);
//        assert_eq!(value_acquire, 42f64);
//    }
//}
//
//#[test]
//fn test_load_order_f32() {
//    let mem = Box::new(42.0f64);
//    let ptr = Box::into_raw(mem);
//    let addr = Address::from_mut_ptr(ptr);
//
//    unsafe {
//        addr.store(10f32);
//
//        let value_relaxed : f32 = addr.load_order(Ordering::Relaxed);
//        assert_eq!(value_relaxed, 10f32);
//
//        let value_seqcst : f32 = addr.load_order(Ordering::SeqCst);
//        assert_eq!(value_seqcst, 10f32);
//
//        let value_acquire : f32 = addr.load_order(Ordering::Acquire);
//        assert_eq!(value_acquire, 10f32);
//    }
//}
