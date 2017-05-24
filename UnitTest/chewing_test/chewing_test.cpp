#include <stdio.h>
#include <stdlib.h>

/*#ifdef __cplusplus
extern "C" {
#endif*/
#include <chewing.h>

/*#ifdef __cplusplus
}
#endif*/


/*
 * By StarForcefield
 * 新酷音輸入法的程式庫libchewing的使用
 */

int selKeys[] = {'1', '2', '3', '4', '5', '6', '7', '8', '9', 0};

int main()
{
  ChewingContext* ct;
  char* buf;
  int counter;

  /*
   * 初始化新酷音輸入法。
   * 這個程式最大的缺點，便是寫死了新酷音輸入法所需資料的來源路徑。
   * 參數1：新酷音輸入法所需資料的來源路徑
   * 參數2：新酷音輸入法學習功能的學習紀錄。該紀錄可以不用存在。
   */
  chewing_Init("/usr/share/chewing", ".");
  ct = chewing_new();

  /* 預設都是中文模式 */
  if(chewing_get_ChiEngMode(ct) == CHINESE_MODE)
    printf("Chinese mode!\n");

  /* 設定選擇候選字的快速鍵。如果不設定就不能選擇候選字的樣子。*/
  chewing_set_selKey(ct, selKeys, 9);
  /* 設定在緩衝區的最大中文字數。如果不設定的話，就不能選字。 */
  chewing_set_maxChiSymbolLen(ct, 10);
  /* 設定選擇字詞的時候，每一頁要列出多少候選字詞 */
  chewing_set_candPerPage(ct, 9);

  /*
   * 序列1：輸入綠茶兩個字。
   * 前六個呼叫是輸入注音符號（ㄌㄩˋㄔㄚˊ）
   * 第七個呼叫是按下Enter的動作。按下Enter後，輸入的文字會從緩衝區送到輸出區。
   */
  chewing_handle_Default(ct, 'x');
  chewing_handle_Default(ct, 'm');
  chewing_handle_Default(ct, '4');
  chewing_handle_Default(ct, 't');
  chewing_handle_Default(ct, '8');
  chewing_handle_Default(ct, '6');
  chewing_handle_Enter(ct);

  /* 自輸出區把字複製出來 */
  buf = chewing_commit_String(ct);
  printf("%s\n", buf);
  free(buf);

  /* ============================
   * 序列2：輸入「ㄓ」，然後啟動選字
   */
  chewing_handle_Default(ct, '5');
  chewing_handle_Space(ct);
  /*
   * 預設的選字按鍵是↓。用這個呼叫來啟動選字
   * 如果不先「按↓」，使用chewing_cand_Enumerate就不會進入列舉的狀態。
   */
  chewing_handle_Down(ct); 

  /* 
   * 開始進行候選字的列舉。
   * 這是一個iterator的架構：
   * 1. 利用chewing_cand_Enumerate初始化ChewingContext中的iterator
   * 2. 利用chewing_cand_hasNext來確認iterator有沒有下一個元素（也就是候選字）
   * 3. 利用chewing_cand_String取出當前元素（候選字）並且將iterator內的指標移動到下一個元素
   */
  chewing_cand_Enumerate(ct);  
  counter = 0;
  while(chewing_cand_hasNext(ct))
    {
      counter += 1;
      char* s = chewing_cand_String(ct);
      printf("%s ", s);
      free(s);
      if (counter == 5)
        {
          counter = 0;
          printf("\n");
        }
    }
  printf("\nSelecting 13rd:");

  /* 
   * 剛才按下了↓，目前正在選字。
   * 我想選第13個字，那就必須換頁，然後選第二頁的第4個字（9+4=13）
   * 換頁的按鍵是空白鍵
   * （一頁有多少候選字的設定，在  chewing_set_candPerPage(ct, 9); 這個呼叫中）
   */
  chewing_handle_Space(ct);
  chewing_handle_Default(ct, '4');
  chewing_handle_Enter(ct);

  buf = chewing_commit_String(ct);
  printf("%s\n", buf);
  free(buf);

  /* 記得收拾殘局的才是好孩子 */
  chewing_delete(ct);
  chewing_Terminate();
  return 0;
}