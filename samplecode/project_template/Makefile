# Dummy makefile, will call the host and enclave makefile when requested.

SRC_U = app/
SRC_T = enclave/

# Compilation process, will call the appropriate makefiles.

all: host enclave

host:
	@echo "\033[32mRequest to compile the host part...\033[0m"
	@make -C $(SRC_U)

enclave:
	@echo "\033[32mRequest to compile the enclave part...\033[0m"
	@make -C $(SRC_T)

clean:
	@make -C $(SRC_U) clean
	@make -C $(SRC_T) clean

fclean:
	@make -C $(SRC_U) fclean
	@make -C $(SRC_T) fclean

clean_host:
	@make -C $(SRC_U) clean

clean_enclave:
	@make -C $(SRC_T) clean

fclean_host:
	@make -C $(SRC_U) fclean

fclean_enclave:
	@make -C $(SRC_T) fclean

re_host: fclean_host host

re_enclave: fclean_enclave enclave

re: fclean all

# Dummy rules to let make know that those rules are not files.

.PHONY: host enclave clean clean_host clean_enclave fclean_host fclean_enclave fclean re re_host re_enclave