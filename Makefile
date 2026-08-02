#
# Makefile
#
# Parameters:
#
# - GAIA_DIR
#

ifndef GAIA_DIR
  $(error `GAIA_DIR` not set)
endif
include $(GAIA_DIR)/src/main/make/Makefile-Rust.mk

# EOF
