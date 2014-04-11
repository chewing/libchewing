TARGET = simple-select

# FIXME: static build and cross-platform support
LIBCHEWING = $(shell dirname $(shell find .. -name libchewing.so))
CHEWING_LDFLAGS = \
	-L $(LIBCHEWING) -lchewing \
	-Wl,-rpath=$(LIBCHEWING)

all: $(TARGET)

$(TARGET): $(TARGET).c
	$(CC) -o $(TARGET) $(TARGET).c \
		-I ../include \
		-D TEST_HASH_DIR="\"$(PWD)\"" \
		-D TEST_DATA_DIR="\"../data\"" \
		$(CHEWING_LDFLAGS)

check: $(TARGET)
	./$(TARGET)
	@which sqlite3 && echo ".dump" | sqlite3 test.sqlite3

clean:
	rm -f $(TARGET) test.sqlite3
