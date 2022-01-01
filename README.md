# tarfs-rs

A very basic FUSE tarfs implementation, currently only supports listing and traversal without links.

## Usage

```shell
./target/release/tarfs-rs
error: The following required arguments were not provided:
    <FILE>
    <MOUNT>

USAGE:
    tarfs-rs <FILE> <MOUNT>
```

## Example

```shell
$ mkdir -p /a/b/c
$ tar cvf a.tar a
a/
a/b/
a/b/c/
$ mkdir mnt
$ ./target/release/tarfs-rs a.tar mnt
$  tree mnt
mnt
└── a
    └── b
        └── c
```

