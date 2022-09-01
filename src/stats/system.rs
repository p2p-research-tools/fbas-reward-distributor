use sysinfo::{System, SystemExt};

pub type SysInfo = (u64, u64, u64, u64, u64, u64);

pub fn init_sysinfo_instance() -> sysinfo::System {
    let mut sys = System::new_all();
    sys.refresh_all();
    sys
}

pub fn get_system_mem_info(sys: &mut System) -> SysInfo {
    (
        get_total_mem_in_gb(sys),
        get_used_mem_in_gb(sys),
        get_available_mem_in_gb(sys),
        get_free_mem_in_gb(sys),
        get_total_swap_in_gb(sys),
        get_used_swap_in_gb(sys),
    )
}

/// not really necessary. more like a sanity check
fn get_total_mem_in_gb(system: &mut System) -> u64 {
    system.total_memory() / 1000000000u64
}

/// The usage is returned in bytes.
fn get_used_mem_in_gb(system: &mut System) -> u64 {
    system.used_memory() / 1000000000u64
}

/// memory available for allocation to new processes
fn get_available_mem_in_gb(system: &mut System) -> u64 {
    system.available_memory() / 1000000000u64
}

/// unused memory in /proc/meminfo. should be small
fn get_free_mem_in_gb(system: &mut System) -> u64 {
    system.free_memory() / 1000000000u64
}

fn get_total_swap_in_gb(system: &mut System) -> u64 {
    system.total_swap() / 1000000000u64
}

fn get_used_swap_in_gb(system: &mut System) -> u64 {
    system.used_swap() / 1000000000u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_memory() {
        let mut sysinfo_instance = init_sysinfo_instance();
        let actual = get_total_mem_in_gb(&mut sysinfo_instance);
        assert!(actual > 0);
    }

    #[test]
    fn used_memory() {
        let mut sysinfo_instance = init_sysinfo_instance();
        let actual = get_used_mem_in_gb(&mut sysinfo_instance);
        assert!(actual <= get_total_mem_in_gb(&mut sysinfo_instance));
    }

    #[test]
    fn available_memory() {
        let mut sysinfo_instance = init_sysinfo_instance();
        let actual = get_available_mem_in_gb(&mut sysinfo_instance);
        assert!(actual <= get_total_mem_in_gb(&mut sysinfo_instance));
    }

    #[test]
    fn free_memory() {
        let mut sysinfo_instance = init_sysinfo_instance();
        let actual = get_free_mem_in_gb(&mut sysinfo_instance);
        assert!(actual <= get_total_mem_in_gb(&mut sysinfo_instance));
    }

    #[test]
    fn total_swap() {
        let mut sysinfo_instance = init_sysinfo_instance();
        let actual = get_total_swap_in_gb(&mut sysinfo_instance);
        assert!(actual > 0);
    }

    #[test]
    fn free_swap() {
        let mut sysinfo_instance = init_sysinfo_instance();
        let actual = get_used_swap_in_gb(&mut sysinfo_instance);
        assert!(actual <= get_total_swap_in_gb(&mut sysinfo_instance));
    }
}
