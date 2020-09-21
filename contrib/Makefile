TARGET = simple-select

# FIXME: static build and cross-platform support
UNAME_S := $(shell uname -s)
# on macOS
ifeq ($(UNAME_S),Darwin)
LIBCHEWING = $(shell dirname $(shell find .. -name libchewing.dylib))
CHEWING_LDFLAGS = \
	-L $(LIBCHEWING) -lchewing \
	-Wl,-rpath,$(LIBCHEWING)
else
LIBCHEWING = $(shell dirname $(shell find .. -name libchewing.so))
CHEWING_LDFLAGS = \
	-L $(LIBCHEWING) -lchewing \
	-Wl,-rpath=$(LIBCHEWING)
endif

all: $(TARGET)

$(TARGET): $(TARGET).c
	$(CC) -o $(TARGET) $(TARGET).c \
		-I ../include \
		-D TEST_HASH_DIR="\"$(PWD)\"" \
		-D TEST_DATA_DIR="\"../data\"" \
		$(CHEWING_LDFLAGS)
ifeq ($(UNAME_S),Darwin)
# relocate to in-tree here dylib if on macOS
	install_name_tool -change /usr/local/lib/libchewing.3.dylib ../src/.libs/libchewing.3.dylib simple-select
endif

check: $(TARGET)
	./$(TARGET)
	@which sqlite3 && echo ".dump" | sqlite3 test.sqlite3

clean:
	rm -f $(TARGET) test.sqlite3
