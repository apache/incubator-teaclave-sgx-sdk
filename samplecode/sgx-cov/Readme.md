# SGX Code Coverage Support

Prerequisite:

1. `lcov`. Install via `sudo apt-get install lcov`

2. Either of `gcov <= 7`, or `llvm-cov >= 11`
- `gcov <= 7`. Install gcc `sudo apt-get install gcc`. For more information around managing multiple gcc/toolchains, please refer to [this article](https://linuxize.com/post/how-to-install-gcc-compiler-on-ubuntu-18-04/).
- `llvm-cov >= 11`. You can either install using apt/yum/dnf, or the official LLVM installation script:

```
wget https://apt.llvm.org/llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh 11
```

If your platform cannot install either of them, you can use another platform to analyze the generated `gcno` and `gcda` files. Ubuntu 18.04 has gcc-7 by default, and can install llvm 11 using the above script.

## One shot

```
$ make COV=1
$ cd bin && ./app && cd ..
$ make gen_cov_html
```

Then open `html/index.html`, where amazing happens!

sgx_cov supports xargo as well:

```
$ XARGO_SGX=1 make COV=1
$ cd bin && ./app && cd ..
$ XARGO_SGX=1 make gen_cov_html
```

## The Magic

* Enable feature `global_exit` for `sgx_urts`
* Inject an `on exit` function using `global_dtors_object!` macro, and invoke `sgx_cov::cov_writeout()`
* `.gcno` would be generated during compile time at `Target_Dir`
* `.gcna` would be generated during run time at `Target_dir`
* `make gen_cov_html` would process `.gcno` and `.gcna` and generate html results.

## More about the magic

To be continued ...
