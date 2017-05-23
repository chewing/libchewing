#include <iostream>
#include <stdio.h>
#include <stdlib.h>
#include <string>

#include "chewing.h"

using namespace std;

int selKeys[] = {'1', '2', '3', '4', '5', '6', '7', '8', '9', 0};

char* chewing_io(){
	/*** Initailize ***/
	ChewingContext* ctx;
	//chewing_Init("/usr/share/chewing", ".");
	ctx = chewing_new();
	
	// Init: Chinese_mode
	if(chewing_get_ChiEngMode(ctx) == CHINESE_MODE)
		cout << "Chinese mode!" << endl;
	
	// Init Select key
	chewing_set_selKey(ctx, selKeys, 9);
	chewing_set_maxChiSymbolLen(ctx, 10);
	chewing_set_candPerPage(ctx, 9);
	
	
	/*string line;
	getline(cin, line);
	while( line != "exit"){
		cout  << line << endl;
		getline(cin, line);
	}
	*/
	
	char ch;
	while( cin.get(ch) ){
		cout << ch;
	}
	
	return NULL;
}

int main(){
	
	chewing_io();
	
	cout << "Finish" << endl;
	return 0;
}
