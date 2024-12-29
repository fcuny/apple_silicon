use apple_silicon::soc::SocInfo;

fn main() {
    let cpu_info = SocInfo::new().unwrap();
    println!(
        "our CPU is an {}, and we have {} CPU cores, and {} GPU cores. The TDP is {}.",
        cpu_info.cpu_brand_name,
        cpu_info.num_cpu_cores,
        cpu_info.num_gpu_cores,
        cpu_info.cpu_max_power.unwrap(),
    );
}
