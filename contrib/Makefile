TARGET = simple-select

all: $(TARGET)

$(TARGET): $(TARGET).c
	$(CC) -o $(TARGET) $(TARGET).c \
		-I ../include \
		-D TEST_HASH_DIR="\"$(PWD)\"" \
		-D TEST_DATA_DIR="\"../data\"" \
		../src/.libs/libchewing.a -lsqlite3

clean:
	rm -f $(TARGET) test.sqlite3
