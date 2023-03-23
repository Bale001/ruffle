package avm2.intrinsics.memory {
    import __ruffle__.stub_method;

    public native function casi32(addr:int, expectedVal:int, newVal:int):int;
    public function mfence():void {
        stub_method("avm2.intrinsics.memory", "mfence");
    }
}