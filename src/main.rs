mod error;
mod soc;

fn main() {
    let cpu_info = soc::SocInfo::new().unwrap();
    println!(
        "our CPU is an {}, and we have {} CPU cores, and {} GPU cores",
        cpu_info.cpu_brand_name, cpu_info.num_cpu_cores, cpu_info.num_gpu_cores,
    );
}
