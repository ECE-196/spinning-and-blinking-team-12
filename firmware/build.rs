#![no_std]      // don't give access to standard library
#![no_main]     // different entry point (non-trivial)


// importing crates for program
use fugit::RateExtU32;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::{
    gpio::{Io, Level, Output},
    timer::timg::TimerGroup,
};
use esp_println::println;

//wtf is all this:
use esp_hal::clock::Clocks;
use esp_hal::mcpwm::PeripheralClockConfig;
use esp_hal::mcpwm::McPwm;
use esp_hal::mcpwm::operator::PwmPinConfig;
use esp_hal::mcpwm::timer::PwmWorkingMode;

// async for concurrency features (executor already started)
#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {      //'_spawner' means its unused
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default());        // take ownership of all peripherals

    let timg0 = TimerGroup::new(peripherals.TIMG0);                     // initialize the 0th timer group (for keeping time)
    esp_hal_embassy::init(timg0.timer0);                                // to provide "monotonics"

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);             // gain control of IO

    let mut led = Output::new(io.pins.gpio18, Level::Low);              // set pin17 initally LOW


    let motor_hi_pin = io.pins.gpio13;
    let motor_lo_pin = io.pins.gpio14;

    let clocks = Clocks::get();
    println!("src clock: {}", clocks.crypto_pwm_clock);                 // verifying speed

    let clock_cfg = PeripheralClockConfig::with_frequency(32.MHz()).unwrap();
    let mut mcpwm = McPwm::new(peripherals.MCPWM0, clock_cfg);
    mcpwm.operator0.set_timer(&mcpwm.timer0);

    

    let (mut motor_hi, mut motor_lo) = mcpwm.operator0.with_pins(
        motor_hi_pin,
        PwmPinConfig::UP_ACTIVE_HIGH,
        motor_lo_pin,
        PwmPinConfig::UP_ACTIVE_HIGH,
    );

    let timer_clock_cfg = clock_cfg
    .timer_clock_with_frequency(u8::MAX as u16, PwmWorkingMode::Increase, 20.kHz())
    .unwrap();

    mcpwm.timer0.start(timer_clock_cfg);

    loop {
        for n in 0..255{
            motor_hi.set_timestamp(n);
            motor_lo.set_timestamp(255-n);
            Timer::after_millis(0_005).await;  
        }
        led.toggle();
        for n in 0..255{
            motor_hi.set_timestamp(255-n);
            motor_lo.set_timestamp(n);
            Timer::after_millis(0_005).await;  
        }
        led.toggle();
    }
}
