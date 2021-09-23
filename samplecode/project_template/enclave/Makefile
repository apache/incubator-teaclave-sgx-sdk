# Makefile settings

APP_T_SIGNED = enclave.signed.so
LIB = ../lib/
BIN = ../bin/
SRC_RST = ./src/
APP_T = enclave.so
NAME_T = libenclave.a
SRC_U = ../app/
SRC_T = ./
OBJ_T = ../obj/
SGX_SDK ?= /opt/sgxsdk
FLAGS = -Wall -Wextra
GCC_STEP1_T = -fstack-protector -I$(CUSTOM_COMMON_PATH)/inc -I$(CUSTOM_EDL_PATH) -I$(SGX_SDK)/include \
	-I$(SGX_SDK)/include/tlibc -I$(SGX_SDK)/include/stlport -I$(SGX_SDK)/include/epid -I $(SRC_T) \
	-L$(LIB) $(ENCLAVE_CFLAGS) $(SGX_COMMON_CFLAGS)
GCC_STEP2_T = -Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles -L$(SGX_LIBRARY_PATH) \
	-Wl,--whole-archive -l$(TRTS_LIB) -Wl,--no-whole-archive \
	-Wl,--start-group -lsgx_tstdc -l$(SERVICE_LIB) -lsgx_tcrypto -L$(LIB) -lenclave -Wl,--end-group \
	-Wl,--version-script=$(SRC_T)Enclave.lds $(ENCLAVE_LDFLAGS)
FILES_T = Enclave_t.c
FILES_T_H = Enclave_t.h
EDL_FILE = Enclave.edl
TOML = Cargo.toml
ENCLAVE_CONFIG = Enclave.config.xml
SGX_ARCH = x64
TRTS_LIB = sgx_trts
SERVICE_LIB = sgx_tservice
ENCLAVE_CARGO_LIB=libsample.a # This is the name of the enclave static library compiled by Cargo/Xargo. You will have to change it, depending of your project.
# Addprefix dependant variables, no need to change those
OUTPUT_T = $(FILES_T:.c=.o)
NAME = $(addprefix $(BIN), $(APP_T_SIGNED))
BIN_T = $(addprefix $(BIN), $(APP_T))
NAME_T_D = $(addprefix $(LIB), $(NAME_T))
OUTPUT_W_FU=$(addprefix $(OBJ_U), $(OUTPUT_U))
FILES_T_F=$(addprefix $(SRC_T), $(FILES_T))
FILES_T_F_RUST=$(addprefix $(SRC_T), $(FILES_T_RUST))
OUTPUT_W_FT=$(addprefix $(OBJ_T), $(OUTPUT_T))
FILES_RUST_F= $(wildcard $(SRC_RST)*.rs) # Wildcard function used, no need to specify the rust files. Safe as we don't compile the rust files with the makefile.

# Contains compilation rules for the enclave part

include ../buildenv.mk

# Custom header files and EDL paths needs to be specified with make (CUSTOM_EDL_PATH) (CUSTOM_COMMON_PATH) Same goes for Xargo location (XARGO_PATH)

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

ifeq ($(MITIGATION-CVE-2020-0551), LOAD)
export MITIGATION_CVE_2020_0551=LOAD
else ifeq ($(MITIGATION-CVE-2020-0551), CF)
export MITIGATION_CVE_2020_0551=CF
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

all: $(NAME)

# We print the compilation mode we're in (hardware/software mode), just as a reminder.

$(NAME): $(BIN_T)
ifeq ($(SGX_MODE), SW)
	@echo "\033[32mSoftware / Simulation mode\033[0m"
else
	@echo "\033[32mHardware mode\033[0m"
endif
	@echo "\033[32mGenerating keys and signing the enclave...\033[0m"
	@mkdir -p $(BIN)
	@openssl genrsa -out Enclave_private.pem -3 3072
	@openssl rsa -in Enclave_private.pem -pubout -out Enclave_public.pem
	@$(SGX_ENCLAVE_SIGNER) sign -key $(SRC_T)Enclave_private.pem -enclave $(BIN_T) -out $@ -config $(SRC_T)Enclave.config.xml

$(BIN_T): $(NAME_T_D)
	@echo "\033[32mBuilding the enclave...\033[0m"
	@$(CXX) $(OUTPUT_W_FT) -o $@ $(GCC_STEP2_T)

$(NAME_T_D): $(FILES_T_F) $(OUTPUT_W_FT) $(FILES_RUST_F) $(EDL_FILE) $(ENCLAVE_CONFIG) $(TOML) # We added as a reference the rust files, along with the EDL, the XML config file and the cargo.toml file, so Make can detect if any change was made
ifeq ($(XARGO_SGX), 1) # Building with Xargo
	@echo "\033[32mBuilding enclave static library with Xargo...\033[0m"
	RUST_TARGET_PATH=$(XARGO_PATH) xargo build --target x86_64-unknown-linux-sgx --release
	@cp ./target/x86_64-unknown-linux-sgx/release/$(ENCLAVE_CARGO_LIB) $(LIB)libenclave.a
else
	@echo "\033[32mBuilding enclave static library with Cargo...\033[0m"
	@cargo build --release
	@cp ./target/release/$(ENCLAVE_CARGO_LIB) $(LIB)libenclave.a
endif

$(FILES_T_F): $(SGX_EDGER8R) $(SRC_T)/Enclave.edl
	@echo "\033[32mGenerating trusted SGX C edl files...\033[0m"
	@$(SGX_EDGER8R) --trusted $(SRC_T)/Enclave.edl --search-path $(SGX_SDK)/include --search-path $(CUSTOM_EDL_PATH) --trusted-dir $(SRC_T)

$(OBJ_T)%.o:$(SRC_T)%.c
	@mkdir -p $(OBJ_T)
	@echo "\033[32m$?: Build in progress...\033[0m"
	@$(CC) $(FLAGS) $(GCC_STEP1_T) -o $@ -c $?

clean: c_clean
	@rm -rf $(OBJ_T)
	@echo "\033[32mObject files deleted\033[0m"

fclean: clean fclean_enclave

fclean_enclave:
	@echo "\033[32mBinary file $(NAME) deleted\033[0m"
	@rm -f $(NAME)
	@rm -f $(BIN_T)
	@rm -f $(LIB)libenclave.a
	@cargo clean && rm -f Cargo.lock

c_clean:
	@echo "\033[32mC edl generated files deleted\033[0m"
	@rm -rf $(FILES_T_F)
	@rm -f $(FILES_T_H)

re: fclean all

.PHONY: all clean c_clean fclean re fclean_enclave
