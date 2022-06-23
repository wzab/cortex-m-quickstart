// Runnable on QEMU ARM

#![no_main]
#![no_std]

//use cortex_m;
//use cortex_m::Peripherals;
use cortex_m::{
   interrupt::{free},
   peripheral::NVIC,
};
use cortex_m_rt::entry;
use cortex_m_semihosting::debug;
use cortex_m_semihosting::hprintln;
extern crate panic_semihosting;
//use panic_halt as _;

use minimult_cortex_m::*;
use stm32f1xx_hal::{
        //clocks::{self, Clocks, InputSrc, PllSrc, Pllp},
    pac,
    pac::{interrupt,Interrupt,TIM1},
    prelude::*,
    timer::{CounterMs, Event},
};

#[entry]
fn main() -> !
{
    hprintln!("To znowu ja!").unwrap();
    let p = pac::Peripherals::take().unwrap();
    let mut flash = p.FLASH.constrain();
    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut mem = Minimult::mem::<[u8; 4096]>();
    let mut mt = Minimult::new(&mut mem, 3);

    let mut q = mt.msgq::<u32>(4);
    let (snd, rcv) = q.ch();

    let sh = mt.share::<u32>(0);
    let shch1 = sh.ch();
    let shch2 = sh.ch();

    mt.register(0/*tid*/, 1, 256, || task0(snd));
    mt.register(1/*tid*/, 1, 256, || task1(rcv, shch1));
    mt.register(2/*tid*/, 1, 256, || task2(shch2));

    // SysTick settings
    /*
    let cmperi = Peripherals::take().unwrap();
    let mut syst = cmperi.SYST;
    syst.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    syst.set_reload(1_000_000);
    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();
    */
    let mut timer = p.TIM1.counter_ms(&clocks);
    timer.start(1.secs()).unwrap();
    timer.listen(Event::Update);
    hprintln!("before ena irq").unwrap();
    hprintln!("after ena irq").unwrap();
    hprintln!("after ena timer").unwrap();

    //NVIC::unpend(pac::Interrupt::TIM2);
   
    // must be error in terms of lifetime and ownership
    //drop(mem);
    //drop(q);
    //drop(snd);
    //drop(rcv);
    //drop(sh);
    //drop(shch1);
    //drop(shch2);
    unsafe {
       cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM1_UP_TIM16);
    }
    hprintln!("Minimult run").unwrap();
    mt.run()
}

#[interrupt]
fn TIM1_UP_TIM16() {
    free(|cs| {
        unsafe { (*pac::TIM1::ptr()).sr.modify(|_, w| w.uif().clear_bit()) }
        Minimult::kick(0/*tid*/);    
        hprintln!("Interrupt").unwrap();
    });
}

/*
#[exception]
fn SysTick()
{
    Minimult::kick(0/*tid*/);
}
*/

fn task0(mut snd: MTMsgSender<u32>)
{
    for vsnd in 0.. {
        Minimult::idle();
        let val2 = vsnd+3;
        hprintln!("task0 send1 {}", vsnd).unwrap();
        snd.send(vsnd);
        hprintln!("task0 send2 {}", val2).unwrap();
        snd.send(val2);
    }
}

fn task1(mut rcv: MTMsgReceiver<u32>, shch: MTSharedCh<u32>)
{
    for i in 0.. {
        let vrcv = rcv.receive();

        //assert_eq!(i, vrcv);
        hprintln!("task1 touch {} {}", vrcv, i).unwrap();
        let mut vtouch = shch.touch();
        *vtouch = vrcv;
    }
}

fn task2(shch: MTSharedCh<u32>)
{
    let mut j = 0;

    while j < 50 {
        let vlook = shch.look();

        //assert!(j <= *vlook);
        //hprintln!("task2 look {}", *vlook).unwrap(); // many lines printed
        j = *vlook;
    }

    hprintln!("task2 exit").unwrap();
    debug::exit(debug::EXIT_SUCCESS);
}

//#[panic_handler]
//fn panic(x: &PanicInfo) -> !{
//   hprintln!("PAnic! {}",x);
//}
