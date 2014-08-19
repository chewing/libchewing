#!/usr/bin/env python
# -*- coding: utf-8 -*-
import chewing

chewing.Init('/usr/share/chewing', '/tmp')
ctx = chewing.ChewingContext()
ctx.Configure(18, 16, 0, 1, 0)
ctx.set_ChiEngMode(1)
ctx.handle_Default(ord("g"))
ctx.handle_Default(ord("j"))
ctx.handle_Space()
print ctx.buffer_String()
ctx.handle_Default(ord("b"))
ctx.handle_Default(ord("j"))
ctx.handle_Default(ord("4"))
ctx.handle_Default(ord("z"))
ctx.handle_Default(ord("8"))
ctx.handle_Default(ord("3"))
ctx.handle_Enter()
print ctx.commit_String()
ctx = None
