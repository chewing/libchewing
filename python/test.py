#!/usr/bin/env python
# -*- coding: utf-8 -*-
import chewing

chewing.Init('/usr/share/chewing', '/tmp')
ctx = chewing.ChewingContext()
ctx.Configure (18, 16, 0, 1, 0);
ctx.set_ChiEngMode(1)
ctx.handle_Default("g")
ctx.handle_Default("j")
ctx.handle_Space()
print ctx.buffer_String()
ctx.handle_Default("b")
ctx.handle_Default("j")
ctx.handle_Default("4")
ctx.handle_Default("z")
ctx.handle_Default("8")
ctx.handle_Default("3")
ctx.handle_Enter()
print ctx.commit_String()
ctx = None
