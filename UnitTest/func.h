#include "chewing.h"
using namespace std;
ChewingContext* convert2ctx( string str){
	ChewingContext* ctx = chewing_new();
	int idx = 0;
	while(str[idx] != '\0'){
		chewing_handle_Default(ctx, str[idx]);
		idx++;
	}
	chewing_handle_Enter(ctx);
	return ctx;
}

ChewingContext* convert2ctx_bychar(char a, char b){
	ChewingContext* ctx = chewing_new();
	chewing_handle_Default(ctx, a);
	chewing_handle_Default(ctx, b);
	chewing_handle_Default(ctx, ' ');
	chewing_handle_Enter(ctx);
	return ctx;
}