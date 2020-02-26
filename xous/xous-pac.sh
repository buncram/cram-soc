#!/bin/sh
for block in $(grep 'unsafe impl Send' betrusted-pac/src/lib.rs | cut -d' ' -f5)
do
    lc=$(echo $block | tr '[A-Z]' '[a-z]')
    echo "
#[allow(unused)]
fn get_$lc() -> Result<&'static betrusted_pac::$lc::RegisterBlock, xous::XousError> {
    let obj = betrusted_pac::$block::ptr();
    xous::rsyscall(xous::SysCall::MapMemory(
        obj as *mut usize,
        obj as *mut usize,
        4096,
        xous::MemoryFlags::R | xous::MemoryFlags::W,
    ))?;
    Ok(unsafe {&*obj})
}"
done
