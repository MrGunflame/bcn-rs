#!/bin/bash

compressonatorcli original.png -fd BC1 bc1
compressonatorcli bc1.dds bc1_decoded.bmp

compressonatorcli original.png -fd BC2 bc2
compressonatorcli bc2.dds bc2_decoded.bmp

compressonatorcli original.png -fd BC3 bc3
compressonatorcli bc3.dds bc3_decoded.bmp

compressonatorcli original.png -fd BC4 bc4
compressonatorcli bc4.dds bc4_decoded.bmp

compressonatorcli original.png -fd BC5 bc5
compressonatorcli bc5.dds bc5_decoded.bmp

compressonatorcli original.png -fd BC6H bc6h
compressonatorcli bc6h.dds bc6h_decoded.bmp

compressonatorcli original.png -fd BC7 bc7
compressonatorcli bc7.dds bc7_decoded.bmp
