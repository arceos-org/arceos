# Useage

## Example

use nvme as block device.

```bash
make disk_img
make A=examples/shell ARCH=x86_64 LOG=info SMP=4 ACCEL=N FEATURES=driver-nvme APP_FEATURES=use-ramfs BLK=y run
```