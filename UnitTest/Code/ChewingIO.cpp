#include <iostream>
#include <stdio.h>
#include <stdlib.h>
#include <string>
#include <sstream>
//#include <curses.h>

#include "chewing.h"

using namespace std;

int selKeys[] = {'1', '2', '3', '4', '5', '6', '7', '8', '9', 0};

string chewing_io( string line){
	/*** Initailize ***/
	ChewingContext* ctx;
	//chewing_Init("/usr/share/chewing", ".");
	ctx = chewing_new();
	
	// Init: Chinese_mode
	/*if(chewing_get_ChiEngMode(ctx) == CHINESE_MODE)
		cout << "Chinese mode!" << endl;*/
	
	// Init Select key
	chewing_set_selKey(ctx, selKeys, 9);
	chewing_set_maxChiSymbolLen(ctx, 10);
	chewing_set_candPerPage(ctx, 9);
	
	char ch;
	int select;
	int ctr;
	string result;
	stringstream sin(line);
	while( sin.get(ch) ){
		switch(ch){
			case ' ':
			case '6':
			case '3':
			case '4':
			case '7':
				chewing_handle_Default(ctx, ch);
				chewing_handle_Down(ctx);
				if( chewing_cand_TotalChoice(ctx) == 0){
					return "";
				}
				chewing_cand_Enumerate(ctx); 
				ctr = 0;
				while(chewing_cand_hasNext(ctx)){
					ctr++;
					cout << ctr << "." << chewing_cand_String(ctx) << "\t";
					if( ctr % 9 == 0){
						cout << endl;
					}
				}
				cout << endl << "Choose: ";
				//cout << "+" << chewing_cand_TotalChoice(ctx) << endl;
				if( !isdigit(cin.peek())){
					cin.ignore(256, '\n');
					return "[Error] select number error.";
				}
				cin >> select;
				cin.ignore(256, '\n');
				if( select <0 || select > chewing_cand_TotalChoice(ctx)){
					return "[Error] select number error.";
				}
				for( int i = 0; i < select / 9; i++){
					chewing_handle_Space(ctx);
				}
				chewing_handle_Default(ctx, (char)( select%9 + 48));
				chewing_handle_Enter(ctx);
				result = result + chewing_commit_String(ctx);
				break;
			//case '\n':
				//chewing_handle_Enter(ctx);
				//cout << chewing_commit_String(ctx);
				//break;
			default:
				//cout << ch;
				chewing_handle_Default(ctx, ch);
		}
	}
	chewing_delete(ctx);
	return result;
}

int main(){
	cout << "[ChewingIO] <E>: enter, <L>: left, <R>: right, <U>: up, <D>: down, <B>: backspace" << endl;
	string line;
	cout << ">";
	while( getline(cin, line) ){
		cout << "[ChewingIO] " << chewing_io( line) << endl;
		cout << ">";
	}
	cout << "Finish" << endl;
	return 0;
}
