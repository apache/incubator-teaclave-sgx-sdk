# Makefile settings - Host part

LIB = ../lib/
BIN = ../bin/
SRC_RST = ./src/
APP_U = app
APP_T = enclave.so
NAME_U = libEnclave_u.a
SRC_U = ./
SRC_T = ../enclave/
OBJ_U = ../obj/
SGX_SDK = /opt/sgxsdk
FLAGS = -Wall -Wextra
GCC_STEP1_U = -I $(SRC_U) -I./include -I$(SGX_SDK)/include -I$(CUSTOM_EDL_PATH) -fPIC -Wno-attributes $(SGX_COMMON_CFLAGS)
FILES_U = Enclave_u.c 
FILES_U_H = Enclave_u.h
BUILD_RS = build.rs
TOML = Cargo.toml 
SGX_ARCH = x64
SGX_SDK = /opt/sgxsdk
TRTS_LIB = sgx_trts
SERVICE_LIB = sgx_tservice
# Addprefix dependant variables, no need to change those
OUTPUT_U = $(FILES_U:.c=.o)
BIN_U = $(addprefix $(BIN), $(APP_U))
NAME_U_D = $(addprefix $(LIB), $(NAME_U))
FILES_U_F=$(addprefix $(SRC_U), $(FILES_U))
OUTPUT_W_FU=$(addprefix $(OBJ_U), $(OUTPUT_U))
FILES_RUST_F= $(wildcard $(SRC_RST)*.rs) # Wildcare function used, no need to specify the rust files. Safe as we don't compile the rust files with the makefile.

# Contains compilation rules for the enclave part

include ../buildenv.mk

# Custom librairies, EDL paths. Needs to be specified with make (CUSTOM_EDL_PATH) (CUSTOM_COMMON_PATH)

# Directly imported from the original Intel SGX samples, helpful to detect the system architecture

ifeq ($(shell getconf LONG_BIT), 32)
	SGX_ARCH := x86
else ifeq ($(findstring -m32, $(CXXFLAGS)), -m32)
	SGX_ARCH := x86
endif

ifeq ($(SGX_ARCH), x86)
	SGX_COMMON_CFLAGS := -m32
	SGX_LIBRARY_PATH := $(SGX_SDK)/lib
	SGX_ENCLAVE_SIGNER := $(SGX_SDK)/bin/x86/sgx_sign
	SGX_EDGER8R := $(SGX_SDK)/bin/x86/sgx_edger8r
else
	SGX_COMMON_CFLAGS := -m64
	SGX_LIBRARY_PATH := $(SGX_SDK)/lib64
	SGX_ENCLAVE_SIGNER := $(SGX_SDK)/bin/x64/sgx_sign
	SGX_EDGER8R := $(SGX_SDK)/bin/x64/sgx_edger8r
endif

# If specified, software / simulation mode. Otherwise, hardware mode no matter what.

ifeq ($(SGX_MODE), SW)
	TRTS_LIB := sgx_trts_sim
	SERVICE_LIB := sgx_tservice_sim
endif

# If debug mode, we can set up extra options such as the debug flags

ifeq ($(SGX_DEBUG), 1)
	SGX_COMMON_CFLAGS += -O0 -g
else
	SGX_COMMON_CFLAGS += -O2
endif

# Compilation process, we set up all the dependencies needed to have the correct order of build, and avoid relink

all: $(BIN_U)

$(FILES_U_F): $(SGX_EDGER8R) $(SRC_T)/Enclave.edl
	@echo "\033[32mGenerating untrusted SGX C edl files...\033[0m"
	@$(SGX_EDGER8R) --untrusted $(SRC_T)/Enclave.edl --search-path $(SGX_SDK)/include --search-path $(CUSTOM_EDL_PATH) --untrusted-dir $(SRC_U)

$(NAME_U_D): $(FILES_U_F) $(OUTPUT_W_FU)
	@echo "\033[32mBuilding untrusted C edl static library...\033[0m"
	@mkdir -p $(LIB)
	@$(AR) rcsD $@ $(OUTPUT_W_FU)

$(OBJ_U)%.o:$(SRC_U)%.c
	@mkdir -p $(OBJ_U)
	@echo "\033[32m$?: Build in progress...\033[0m"
	@$(CC) $(FLAGS) $(GCC_STEP1_U) -o $@ -c $?

# We print the compilation mode we're in (hardware/software mode), just as a reminder.

$(BIN_U): $(NAME_U_D) $(FILES_RUST_F) $(FILES_U_H) $(BUILD_RS) $(TOML) # We added as a reference the rust files, along with the build.rs file and the cargo.toml file, so Make can detect if any change was made
ifeq ($(SGX_MODE), SW)
	@echo "\033[32mSoftware / Simulation mode\033[0m"
else
	@echo "\033[32mHardware mode\033[0m"
endif
	@echo "\033[32mStarting cargo to build the host...\033[0m"
	@cd $(SRC_U) && SGX_SDK=$(SGX_SDK) cargo build --release
	@echo "\033[32mCopying the host to the correct location... ($(BIN_U))\033[0m"
	@mkdir -p $(BIN)
	@cp $(SRC_U)/target/release/app $(BIN)

clean: c_clean
	@rm -rf $(OBJ_U)
	@echo "\033[32mObject files deleted\033[0m"

fclean: clean fclean_host

fclean_host:
	@echo "\033[32mBinary file $(BIN_U) deleted\033[0m"
	@rm -f $(BIN_U)
	@rm -f $(NAME_U_D)
	@cargo clean && rm -f Cargo.lock


c_clean:
	@echo "\033[32mC edl generated files deleted\033[0m"
	@rm -rf $(FILES_U_F)
	@rm -rf $(FILES_U_H)

re: fclean all

.PHONY: all clean c_clean fclean re fclean_host