/// All CPU memory access goes through this trait.
/// Implementations can intercept, record, or redirect operations.
/// `&mut self` on both methods enables future TracingBus without RefCell.
pub trait Bus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);

    /// A read of `IE` ($FFFF) or `IF` ($FF0F) by the CPU's interrupt logic (HALT, the wake
    /// check, dispatch). On hardware these registers are wired to the interrupt logic and the
    /// access takes no bus cycle, so a recording bus can leave it out of its log.
    /// Defaults to [`Bus::read`].
    fn read_interrupt_register(&mut self, addr: u16) -> u8 {
        self.read(addr)
    }

    /// The interrupt logic's write of `IF` when it acknowledges a request; like
    /// [`Bus::read_interrupt_register`], no bus cycle. Defaults to [`Bus::write`].
    fn write_interrupt_register(&mut self, addr: u16, value: u8) {
        self.write(addr, value);
    }
}
