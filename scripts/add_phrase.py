# -*- coding: utf-8 -*-
if __name__=="__main__":
  f = open("../data/tsi.src","r+")
  f2= open("add_phrase.txt","r")
  article = f.read()
  f.seek(0,2)
  for add in f2:
    phrase = add.split()[0]
    if article.find(phrase)<0:
      f.write(add)
  f.seek(0)
  list = [ line for line in f ]
  list.sort()
  f.seek(0)
  for element in list:
    f.write(element)
  f.close  
