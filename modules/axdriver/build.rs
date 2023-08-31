const NET_DEV_FEATURES: &[&str] = &["ixgbe", "virtio-net"];
const BLOCK_DEV_FEATURES: &[&str] = &["ramdisk", "bcm2835-sdhci", "virtio-blk"];
const DISPLAY_DEV_FEATURES: &[&str] = &["virtio-gpu"];

fn has_feature(feature: &str) -> bool {
    std::env::var(format!(
        "CARGO_FEATURE_{}",
        feature.to_uppercase().replace('-', "_")
    ))
    .is_ok()
}

fn enable_cfg(key: &str, value: &str) {
    println!("cargo:rustc-cfg={key}=\"{value}\"");
}

fn main() {
    if has_feature("bus-pci") {
        enable_cfg("bus", "pci");
    } else {
        enable_cfg("bus", "mmio");
    }

    // Generate cfgs like `net_dev="virtio-net"`. if `dyn` is not enabled, only one device is
    // selected for each device category. If no device is selected, `dummy` is selected.
    let is_dyn = has_feature("dyn");
    for (dev_kind, feat_list) in [
        ("net", NET_DEV_FEATURES),
        ("block", BLOCK_DEV_FEATURES),
        ("display", DISPLAY_DEV_FEATURES),
    ] {
        if !has_feature(dev_kind) {
            continue;
        }

        let mut selected = false;
        for feat in feat_list {
            if has_feature(feat) {
                enable_cfg(&format!("{dev_kind}_dev"), feat);
                selected = true;
                if !is_dyn {
                    break;
                }
            }
        }
        if !is_dyn && !selected {
            enable_cfg(&format!("{dev_kind}_dev"), "dummy");
        }
    }
}
