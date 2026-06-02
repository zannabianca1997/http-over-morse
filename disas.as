
target/avr-none/release/wiretap.elf:     file format elf32-avr


Disassembly of section .text:

00000000 <__vectors>:
   0:	0c 94 34 00 	jmp	0x68	; 0x68 <__ctors_end>
   4:	0c 94 e4 01 	jmp	0x3c8	; 0x3c8 <__vector_1>
   8:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
   c:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  10:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  14:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  18:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  1c:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  20:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  24:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  28:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  2c:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  30:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  34:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  38:	0c 94 34 02 	jmp	0x468	; 0x468 <__vector_14>
  3c:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  40:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  44:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  48:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  4c:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  50:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  54:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  58:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  5c:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  60:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>
  64:	0c 94 4b 00 	jmp	0x96	; 0x96 <__bad_interrupt>

00000068 <__ctors_end>:
  68:	11 24       	eor	r1, r1
  6a:	1f be       	out	0x3f, r1	; 63

0000006c <__do_copy_data>:
  6c:	11 e0       	ldi	r17, 0x01	; 1

0000006e <.Loc.1>:
  6e:	a0 e0       	ldi	r26, 0x00	; 0

00000070 <.Loc.2>:
  70:	b1 e0       	ldi	r27, 0x01	; 1

00000072 <.Loc.3>:
  72:	ee e0       	ldi	r30, 0x0E	; 14

00000074 <.Loc.4>:
  74:	f5 e0       	ldi	r31, 0x05	; 5

00000076 <.Loc.5>:
  76:	02 c0       	rjmp	.+4      	; 0x7c <.L__do_copy_data_start>

00000078 <.L__do_copy_data_loop>:
  78:	05 90       	lpm	r0, Z+

0000007a <.Loc.7>:
  7a:	0d 92       	st	X+, r0

0000007c <.L__do_copy_data_start>:
  7c:	ac 33       	cpi	r26, 0x3C	; 60

0000007e <.Loc.9>:
  7e:	b1 07       	cpc	r27, r17

00000080 <.Loc.10>:
  80:	d9 f7       	brne	.-10     	; 0x78 <.L__do_copy_data_loop>

00000082 <__do_clear_bss>:
  82:	21 e0       	ldi	r18, 0x01	; 1

00000084 <.Loc.1>:
  84:	ac e3       	ldi	r26, 0x3C	; 60

00000086 <.Loc.2>:
  86:	b1 e0       	ldi	r27, 0x01	; 1

00000088 <.Loc.3>:
  88:	01 c0       	rjmp	.+2      	; 0x8c <.Loc.5>

0000008a <.Loc.4>:
  8a:	1d 92       	st	X+, r1

0000008c <.Loc.5>:
  8c:	a3 35       	cpi	r26, 0x53	; 83

0000008e <.Loc.6>:
  8e:	b2 07       	cpc	r27, r18

00000090 <.Loc.7>:
  90:	e1 f7       	brne	.-8      	; 0x8a <.Loc.4>

00000092 <main>:
  92:	0e 94 4d 00 	call	0x9a	; 0x9a <_RNvCsauX2dqpUjbr_7wiretap20___avr_device_rt_main>

00000096 <__bad_interrupt>:
  96:	0c 94 00 00 	jmp	0	; 0x0 <__vectors>

0000009a <_RNvCsauX2dqpUjbr_7wiretap20___avr_device_rt_main>:
  9a:	2f 92       	push	r2
  9c:	3f 92       	push	r3
  9e:	4f 92       	push	r4
  a0:	5f 92       	push	r5
  a2:	6f 92       	push	r6
  a4:	7f 92       	push	r7
  a6:	8f 92       	push	r8
  a8:	9f 92       	push	r9
  aa:	af 92       	push	r10
  ac:	bf 92       	push	r11
  ae:	cf 92       	push	r12
  b0:	df 92       	push	r13
  b2:	ef 92       	push	r14
  b4:	ff 92       	push	r15
  b6:	0f 93       	push	r16
  b8:	1f 93       	push	r17
  ba:	11 e0       	ldi	r17, 0x01	; 1
  bc:	10 93 3c 01 	sts	0x013C, r17	; 0x80013c <DEVICE_PERIPHERALS>
  c0:	59 98       	cbi	0x0b, 1	; 11
  c2:	51 9a       	sbi	0x0a, 1	; 10
  c4:	8f ec       	ldi	r24, 0xCF	; 207
  c6:	90 e0       	ldi	r25, 0x00	; 0
  c8:	90 93 c5 00 	sts	0x00C5, r25	; 0x8000c5 <__TEXT_REGION_LENGTH__+0x7f80c5>
  cc:	80 93 c4 00 	sts	0x00C4, r24	; 0x8000c4 <__TEXT_REGION_LENGTH__+0x7f80c4>
  d0:	82 e0       	ldi	r24, 0x02	; 2
  d2:	80 93 c0 00 	sts	0x00C0, r24	; 0x8000c0 <__TEXT_REGION_LENGTH__+0x7f80c0>
  d6:	98 e1       	ldi	r25, 0x18	; 24
  d8:	90 93 c1 00 	sts	0x00C1, r25	; 0x8000c1 <__TEXT_REGION_LENGTH__+0x7f80c1>
  dc:	96 e0       	ldi	r25, 0x06	; 6
  de:	90 93 c2 00 	sts	0x00C2, r25	; 0x8000c2 <__TEXT_REGION_LENGTH__+0x7f80c2>
  e2:	84 bd       	out	0x24, r24	; 36
  e4:	9d e7       	ldi	r25, 0x7D	; 125
  e6:	97 bd       	out	0x27, r25	; 39
  e8:	95 e0       	ldi	r25, 0x05	; 5
  ea:	95 bd       	out	0x25, r25	; 37
  ec:	80 93 6e 00 	sts	0x006E, r24	; 0x80006e <__TEXT_REGION_LENGTH__+0x7f806e>
  f0:	10 93 69 00 	sts	0x0069, r17	; 0x800069 <__TEXT_REGION_LENGTH__+0x7f8069>
  f4:	1d bb       	out	0x1d, r17	; 29
  f6:	8f b7       	in	r24, 0x3f	; 63
  f8:	f8 94       	cli
  fa:	8f bf       	out	0x3f, r24	; 63
  fc:	8f b7       	in	r24, 0x3f	; 63
  fe:	f8 94       	cli
 100:	8f bf       	out	0x3f, r24	; 63
 102:	78 94       	sei
 104:	e0 e0       	ldi	r30, 0x00	; 0
 106:	f0 e0       	ldi	r31, 0x00	; 0
 108:	8f e7       	ldi	r24, 0x7F	; 127
 10a:	90 e0       	ldi	r25, 0x00	; 0
 10c:	4c 01       	movw	r8, r24
 10e:	8b ea       	ldi	r24, 0xAB	; 171
 110:	9a e2       	ldi	r25, 0x2A	; 42
 112:	2c 01       	movw	r4, r24
 114:	31 2d       	mov	r19, r1
 116:	5f 01       	movw	r10, r30
 118:	21 2d       	mov	r18, r1
 11a:	81 2d       	mov	r24, r1
 11c:	df 01       	movw	r26, r30
 11e:	f4 2e       	mov	r15, r20
 120:	31 70       	andi	r19, 0x01	; 1
 122:	30 30       	cpi	r19, 0x00	; 0
 124:	81 f0       	breq	.+32     	; 0x146 <.Lname55+0x2>
 126:	90 91 c0 00 	lds	r25, 0x00C0	; 0x8000c0 <__TEXT_REGION_LENGTH__+0x7f80c0>
 12a:	90 72       	andi	r25, 0x20	; 32
 12c:	31 e0       	ldi	r19, 0x01	; 1
 12e:	90 30       	cpi	r25, 0x00	; 0
 130:	4f 2d       	mov	r20, r15
 132:	6b 01       	movw	r12, r22
 134:	fd 01       	movw	r30, r26
 136:	91 f3       	breq	.-28     	; 0x11c <.Lname47+0x4>
 138:	f0 92 c6 00 	sts	0x00C6, r15	; 0x8000c6 <__TEXT_REGION_LENGTH__+0x7f80c6>
 13c:	4f 2d       	mov	r20, r15
 13e:	31 2d       	mov	r19, r1
 140:	b6 01       	movw	r22, r12
 142:	fd 01       	movw	r30, r26
 144:	eb cf       	rjmp	.-42     	; 0x11c <.Lname47+0x4>
 146:	a1 15       	cp	r26, r1
 148:	b1 05       	cpc	r27, r1
 14a:	09 f0       	breq	.+2      	; 0x14e <.Lname56+0x4>
 14c:	d8 c0       	rjmp	.+432    	; 0x2fe <.Lname87+0xfb>
 14e:	81 70       	andi	r24, 0x01	; 1
 150:	31 e0       	ldi	r19, 0x01	; 1
 152:	40 e2       	ldi	r20, 0x20	; 32
 154:	80 30       	cpi	r24, 0x00	; 0
 156:	1b 01       	movw	r2, r22
 158:	f5 01       	movw	r30, r10
 15a:	81 2d       	mov	r24, r1
 15c:	f9 f6       	brne	.-66     	; 0x11c <.Lname47+0x4>
 15e:	02 2f       	mov	r16, r18
 160:	65 01       	movw	r12, r10
 162:	b4 01       	movw	r22, r8
 164:	4f b7       	in	r20, 0x3f	; 63
 166:	f8 94       	cli
 168:	20 91 4f 01 	lds	r18, 0x014F	; 0x80014f <_RNvNtNtCsbOh4P07NHjy_14layer_physical9from_wire16wire_to_segments6BUFFER+0x12>
 16c:	30 91 50 01 	lds	r19, 0x0150	; 0x800150 <_RNvNtNtCsbOh4P07NHjy_14layer_physical9from_wire16wire_to_segments6BUFFER+0x13>
 170:	e0 91 4d 01 	lds	r30, 0x014D	; 0x80014d <_RNvNtNtCsbOh4P07NHjy_14layer_physical9from_wire16wire_to_segments6BUFFER+0x10>
 174:	f0 91 4e 01 	lds	r31, 0x014E	; 0x80014e <_RNvNtNtCsbOh4P07NHjy_14layer_physical9from_wire16wire_to_segments6BUFFER+0x11>
 178:	4f bf       	out	0x3f, r20	; 63
 17a:	e2 17       	cp	r30, r18
 17c:	f3 07       	cpc	r31, r19
 17e:	89 f0       	breq	.+34     	; 0x1a2 <.Lname70>
 180:	df 01       	movw	r26, r30
 182:	af 70       	andi	r26, 0x0F	; 15
 184:	b0 70       	andi	r27, 0x00	; 0
 186:	a3 5c       	subi	r26, 0xC3	; 195
 188:	be 4f       	sbci	r27, 0xFE	; 254
 18a:	4c 91       	ld	r20, X
 18c:	2f b7       	in	r18, 0x3f	; 63
 18e:	f8 94       	cli
 190:	31 96       	adiw	r30, 0x01	; 1
 192:	f0 93 4e 01 	sts	0x014E, r31	; 0x80014e <_RNvNtNtCsbOh4P07NHjy_14layer_physical9from_wire16wire_to_segments6BUFFER+0x11>
 196:	e0 93 4d 01 	sts	0x014D, r30	; 0x80014d <_RNvNtNtCsbOh4P07NHjy_14layer_physical9from_wire16wire_to_segments6BUFFER+0x10>
 19a:	2f bf       	out	0x3f, r18	; 63
 19c:	40 30       	cpi	r20, 0x00	; 0
 19e:	09 f0       	breq	.+2      	; 0x1a2 <.Lname70>
 1a0:	19 c0       	rjmp	.+50     	; 0x1d4 <.Lname79+0x1>
 1a2:	4f b7       	in	r20, 0x3f	; 63
 1a4:	f8 94       	cli
 1a6:	20 91 51 01 	lds	r18, 0x0151	; 0x800151 <_RNvNtCsj8pYqV1fK8l_4time9stopwatch9STOPWATCH.0>
 1aa:	30 91 52 01 	lds	r19, 0x0152	; 0x800152 <_RNvNtCsj8pYqV1fK8l_4time9stopwatch9STOPWATCH.0+0x1>
 1ae:	28 30       	cpi	r18, 0x08	; 8
 1b0:	31 05       	cpc	r19, r1
 1b2:	08 f4       	brcc	.+2      	; 0x1b6 <.Lname73+0x3>
 1b4:	00 c1       	rjmp	.+512    	; 0x3b6 <.Lname87+0x1b3>
 1b6:	d0 92 52 01 	sts	0x0152, r13	; 0x800152 <_RNvNtCsj8pYqV1fK8l_4time9stopwatch9STOPWATCH.0+0x1>
 1ba:	c0 92 51 01 	sts	0x0151, r12	; 0x800151 <_RNvNtCsj8pYqV1fK8l_4time9stopwatch9STOPWATCH.0>
 1be:	4f bf       	out	0x3f, r20	; 63
 1c0:	2f 37       	cpi	r18, 0x7F	; 127
 1c2:	31 05       	cpc	r19, r1
 1c4:	08 f0       	brcs	.+2      	; 0x1c8 <.Lname76+0x4>
 1c6:	9b 01       	movw	r18, r22
 1c8:	49 b1       	in	r20, 0x09	; 9
 1ca:	42 95       	swap	r20
 1cc:	40 7f       	andi	r20, 0xF0	; 240
 1ce:	44 0f       	add	r20, r20
 1d0:	40 78       	andi	r20, 0x80	; 128
 1d2:	42 2b       	or	r20, r18
 1d4:	34 2f       	mov	r19, r20
 1d6:	3f 77       	andi	r19, 0x7F	; 127
 1d8:	14 16       	cp	r1, r20
 1da:	a4 f4       	brge	.+40     	; 0x204 <.Lname87+0x1>
 1dc:	21 2f       	mov	r18, r17
 1de:	21 70       	andi	r18, 0x01	; 1
 1e0:	20 30       	cpi	r18, 0x00	; 0
 1e2:	09 f0       	breq	.+2      	; 0x1e6 <.Lname82+0x2>
 1e4:	bf cf       	rjmp	.-130    	; 0x164 <.Lname59+0x5>
 1e6:	32 30       	cpi	r19, 0x02	; 2
 1e8:	08 f4       	brcc	.+2      	; 0x1ec <.Lname83+0x1>
 1ea:	bc cf       	rjmp	.-136    	; 0x164 <.Lname59+0x5>
 1ec:	09 30       	cpi	r16, 0x09	; 9
 1ee:	08 f0       	brcs	.+2      	; 0x1f2 <.Lname84>
 1f0:	79 c0       	rjmp	.+242    	; 0x2e4 <.Lname87+0xe1>
 1f2:	49 ea       	ldi	r20, 0xA9	; 169
 1f4:	51 e0       	ldi	r21, 0x01	; 1
 1f6:	80 2f       	mov	r24, r16
 1f8:	8a 95       	dec	r24
 1fa:	0a f4       	brpl	.+2      	; 0x1fe <.Lname86>
 1fc:	6d c0       	rjmp	.+218    	; 0x2d8 <.Lname87+0xd5>
 1fe:	56 95       	lsr	r21
 200:	47 95       	ror	r20
 202:	fa cf       	rjmp	.-12     	; 0x1f8 <.Lname84+0x6>
 204:	11 2d       	mov	r17, r1
 206:	32 30       	cpi	r19, 0x02	; 2
 208:	21 2f       	mov	r18, r17
 20a:	08 f0       	brcs	.+2      	; 0x20e <.Lname87+0xb>
 20c:	20 e8       	ldi	r18, 0x80	; 128
 20e:	02 2b       	or	r16, r18
 210:	26 e4       	ldi	r18, 0x46	; 70
 212:	02 02       	muls	r16, r18
 214:	40 2d       	mov	r20, r0
 216:	11 24       	eor	r1, r1
 218:	21 ef       	ldi	r18, 0xF1	; 241
 21a:	62 2f       	mov	r22, r18
 21c:	06 9f       	mul	r16, r22
 21e:	20 2d       	mov	r18, r0
 220:	31 2d       	mov	r19, r1
 222:	11 24       	eor	r1, r1
 224:	34 0f       	add	r19, r20
 226:	40 2f       	mov	r20, r16
 228:	55 27       	eor	r21, r21
 22a:	56 02       	muls	r21, r22
 22c:	11 24       	eor	r1, r1
 22e:	40 2d       	mov	r20, r0
 230:	43 0f       	add	r20, r19
 232:	55 27       	eor	r21, r21
 234:	54 2f       	mov	r21, r20
 236:	44 27       	eor	r20, r20
 238:	62 2e       	mov	r6, r18
 23a:	77 24       	eor	r7, r7
 23c:	64 2a       	or	r6, r20
 23e:	75 2a       	or	r7, r21
 240:	b3 01       	movw	r22, r6
 242:	c6 01       	movw	r24, r12
 244:	21 e8       	ldi	r18, 0x81	; 129
 246:	38 e8       	ldi	r19, 0x88	; 136
 248:	a6 01       	movw	r20, r12
 24a:	0e 94 5d 02 	call	0x4ba	; 0x4ba <__mulsi3>
 24e:	81 2d       	mov	r24, r1
 250:	99 0f       	add	r25, r25
 252:	28 2f       	mov	r18, r24
 254:	22 1f       	adc	r18, r18
 256:	99 0f       	add	r25, r25
 258:	22 1f       	adc	r18, r18
 25a:	98 e7       	ldi	r25, 0x78	; 120
 25c:	29 02       	muls	r18, r25
 25e:	90 2d       	mov	r25, r0
 260:	11 24       	eor	r1, r1
 262:	37 e0       	ldi	r19, 0x07	; 7
 264:	43 2f       	mov	r20, r19
 266:	24 9f       	mul	r18, r20
 268:	20 2d       	mov	r18, r0
 26a:	31 2d       	mov	r19, r1
 26c:	11 24       	eor	r1, r1
 26e:	39 2b       	or	r19, r25
 270:	84 02       	muls	r24, r20
 272:	11 24       	eor	r1, r1
 274:	80 2d       	mov	r24, r0
 276:	83 0f       	add	r24, r19
 278:	99 27       	eor	r25, r25
 27a:	98 2f       	mov	r25, r24
 27c:	88 27       	eor	r24, r24
 27e:	33 27       	eor	r19, r19
 280:	28 2b       	or	r18, r24
 282:	39 2b       	or	r19, r25
 284:	62 1a       	sub	r6, r18
 286:	73 0a       	sbc	r7, r19
 288:	b3 01       	movw	r22, r6
 28a:	c6 01       	movw	r24, r12
 28c:	92 01       	movw	r18, r4
 28e:	a6 01       	movw	r20, r12
 290:	0e 94 5d 02 	call	0x4ba	; 0x4ba <__mulsi3>
 294:	96 95       	lsr	r25
 296:	87 95       	ror	r24
 298:	96 95       	lsr	r25
 29a:	87 95       	ror	r24
 29c:	96 95       	lsr	r25
 29e:	87 95       	ror	r24
 2a0:	20 e3       	ldi	r18, 0x30	; 48
 2a2:	92 02       	muls	r25, r18
 2a4:	90 2d       	mov	r25, r0
 2a6:	11 24       	eor	r1, r1
 2a8:	82 9f       	mul	r24, r18
 2aa:	81 2d       	mov	r24, r1
 2ac:	11 24       	eor	r1, r1
 2ae:	89 0f       	add	r24, r25
 2b0:	99 27       	eor	r25, r25
 2b2:	98 2f       	mov	r25, r24
 2b4:	88 27       	eor	r24, r24
 2b6:	20 2d       	mov	r18, r0
 2b8:	33 27       	eor	r19, r19
 2ba:	28 2b       	or	r18, r24
 2bc:	39 2b       	or	r19, r25
 2be:	c3 01       	movw	r24, r6
 2c0:	82 1b       	sub	r24, r18
 2c2:	93 0b       	sbc	r25, r19
 2c4:	80 50       	subi	r24, 0x00	; 0
 2c6:	9f 4f       	sbci	r25, 0xFF	; 255
 2c8:	dc 01       	movw	r26, r24
 2ca:	8c 91       	ld	r24, X
 2cc:	8f 3f       	cpi	r24, 0xFF	; 255
 2ce:	21 2f       	mov	r18, r17
 2d0:	09 f4       	brne	.+2      	; 0x2d4 <.Lname87+0xd1>
 2d2:	45 cf       	rjmp	.-374    	; 0x15e <.Lname58+0x6>
 2d4:	28 2f       	mov	r18, r24
 2d6:	43 cf       	rjmp	.-378    	; 0x15e <.Lname58+0x6>
 2d8:	41 70       	andi	r20, 0x01	; 1
 2da:	11 e0       	ldi	r17, 0x01	; 1
 2dc:	40 30       	cpi	r20, 0x00	; 0
 2de:	21 2d       	mov	r18, r1
 2e0:	09 f0       	breq	.+2      	; 0x2e4 <.Lname87+0xe1>
 2e2:	3d cf       	rjmp	.-390    	; 0x15e <.Lname58+0x6>
 2e4:	81 e0       	ldi	r24, 0x01	; 1
 2e6:	35 30       	cpi	r19, 0x05	; 5
 2e8:	08 f4       	brcc	.+2      	; 0x2ec <.Lname87+0xe9>
 2ea:	81 2d       	mov	r24, r1
 2ec:	01 34       	cpi	r16, 0x41	; 65
 2ee:	b1 01       	movw	r22, r2
 2f0:	20 2f       	mov	r18, r16
 2f2:	a4 f0       	brlt	.+40     	; 0x31c <.Lname87+0x119>
 2f4:	21 54       	subi	r18, 0x41	; 65
 2f6:	2a 31       	cpi	r18, 0x1A	; 26
 2f8:	08 f4       	brcc	.+2      	; 0x2fc <.Lname87+0xf9>
 2fa:	22 c0       	rjmp	.+68     	; 0x340 <.Lname87+0x13d>
 2fc:	63 c0       	rjmp	.+198    	; 0x3c4 <.Lname87+0x1c1>
 2fe:	31 e0       	ldi	r19, 0x01	; 1
 300:	4d e5       	ldi	r20, 0x5D	; 93
 302:	61 15       	cp	r22, r1
 304:	7b 01       	movw	r14, r22
 306:	71 05       	cpc	r23, r1
 308:	f5 01       	movw	r30, r10
 30a:	bf 01       	movw	r22, r30
 30c:	09 f4       	brne	.+2      	; 0x310 <.Lname87+0x10d>
 30e:	06 cf       	rjmp	.-500    	; 0x11c <.Lname47+0x4>
 310:	4d 91       	ld	r20, X+
 312:	f7 01       	movw	r30, r14
 314:	31 97       	sbiw	r30, 0x01	; 1
 316:	bf 01       	movw	r22, r30
 318:	fd 01       	movw	r30, r26
 31a:	00 cf       	rjmp	.-512    	; 0x11c <.Lname47+0x4>
 31c:	09 31       	cpi	r16, 0x19	; 25
 31e:	44 f0       	brlt	.+16     	; 0x330 <.Lname87+0x12d>
 320:	20 53       	subi	r18, 0x30	; 48
 322:	2a 30       	cpi	r18, 0x0A	; 10
 324:	68 f0       	brcs	.+26     	; 0x340 <.Lname87+0x13d>
 326:	09 31       	cpi	r16, 0x19	; 25
 328:	59 f0       	breq	.+22     	; 0x340 <.Lname87+0x13d>
 32a:	00 32       	cpi	r16, 0x20	; 32
 32c:	49 f0       	breq	.+18     	; 0x340 <.Lname87+0x13d>
 32e:	4a c0       	rjmp	.+148    	; 0x3c4 <.Lname87+0x1c1>
 330:	2a 95       	dec	r18
 332:	22 30       	cpi	r18, 0x02	; 2
 334:	28 f0       	brcs	.+10     	; 0x340 <.Lname87+0x13d>
 336:	04 30       	cpi	r16, 0x04	; 4
 338:	19 f0       	breq	.+6      	; 0x340 <.Lname87+0x13d>
 33a:	06 30       	cpi	r16, 0x06	; 6
 33c:	09 f0       	breq	.+2      	; 0x340 <.Lname87+0x13d>
 33e:	42 c0       	rjmp	.+132    	; 0x3c4 <.Lname87+0x1c1>
 340:	20 2f       	mov	r18, r16
 342:	21 54       	subi	r18, 0x41	; 65
 344:	31 e0       	ldi	r19, 0x01	; 1
 346:	2a 31       	cpi	r18, 0x1A	; 26
 348:	40 2f       	mov	r20, r16
 34a:	f5 01       	movw	r30, r10
 34c:	21 2d       	mov	r18, r1
 34e:	11 e0       	ldi	r17, 0x01	; 1
 350:	08 f4       	brcc	.+2      	; 0x354 <.Lname87+0x151>
 352:	e4 ce       	rjmp	.-568    	; 0x11c <.Lname47+0x4>
 354:	20 2f       	mov	r18, r16
 356:	20 53       	subi	r18, 0x30	; 48
 358:	2a 30       	cpi	r18, 0x0A	; 10
 35a:	40 2f       	mov	r20, r16
 35c:	b1 01       	movw	r22, r2
 35e:	f5 01       	movw	r30, r10
 360:	21 2d       	mov	r18, r1
 362:	08 f4       	brcc	.+2      	; 0x366 <.Lname87+0x163>
 364:	db ce       	rjmp	.-586    	; 0x11c <.Lname47+0x4>
 366:	00 32       	cpi	r16, 0x20	; 32
 368:	40 2f       	mov	r20, r16
 36a:	b1 01       	movw	r22, r2
 36c:	f5 01       	movw	r30, r10
 36e:	21 2d       	mov	r18, r1
 370:	09 f4       	brne	.+2      	; 0x374 <.Lname87+0x171>
 372:	d4 ce       	rjmp	.-600    	; 0x11c <.Lname47+0x4>
 374:	04 30       	cpi	r16, 0x04	; 4
 376:	5c f0       	brlt	.+22     	; 0x38e <.Lname87+0x18b>
 378:	04 30       	cpi	r16, 0x04	; 4
 37a:	81 f0       	breq	.+32     	; 0x39c <.Lname87+0x199>
 37c:	06 30       	cpi	r16, 0x06	; 6
 37e:	89 f0       	breq	.+34     	; 0x3a2 <.Lname87+0x19f>
 380:	09 31       	cpi	r16, 0x19	; 25
 382:	09 f0       	breq	.+2      	; 0x386 <.Lname87+0x183>
 384:	1f c0       	rjmp	.+62     	; 0x3c4 <.Lname87+0x1c1>
 386:	e6 e3       	ldi	r30, 0x36	; 54
 388:	f1 e0       	ldi	r31, 0x01	; 1
 38a:	63 e0       	ldi	r22, 0x03	; 3
 38c:	10 c0       	rjmp	.+32     	; 0x3ae <.Lname87+0x1ab>
 38e:	01 30       	cpi	r16, 0x01	; 1
 390:	59 f0       	breq	.+22     	; 0x3a8 <.Lname87+0x1a5>
 392:	02 30       	cpi	r16, 0x02	; 2
 394:	b9 f4       	brne	.+46     	; 0x3c4 <.Lname87+0x1c1>
 396:	e4 e3       	ldi	r30, 0x34	; 52
 398:	f1 e0       	ldi	r31, 0x01	; 1
 39a:	08 c0       	rjmp	.+16     	; 0x3ac <.Lname87+0x1a9>
 39c:	e9 e3       	ldi	r30, 0x39	; 57
 39e:	f1 e0       	ldi	r31, 0x01	; 1
 3a0:	05 c0       	rjmp	.+10     	; 0x3ac <.Lname87+0x1a9>
 3a2:	e0 e3       	ldi	r30, 0x30	; 48
 3a4:	f1 e0       	ldi	r31, 0x01	; 1
 3a6:	02 c0       	rjmp	.+4      	; 0x3ac <.Lname87+0x1a9>
 3a8:	e2 e3       	ldi	r30, 0x32	; 50
 3aa:	f1 e0       	ldi	r31, 0x01	; 1
 3ac:	62 e0       	ldi	r22, 0x02	; 2
 3ae:	70 e0       	ldi	r23, 0x00	; 0
 3b0:	4b e5       	ldi	r20, 0x5B	; 91
 3b2:	21 2d       	mov	r18, r1
 3b4:	b3 ce       	rjmp	.-666    	; 0x11c <.Lname47+0x4>
 3b6:	4f bf       	out	0x3f, r20	; 63
 3b8:	4f 2d       	mov	r20, r15
 3ba:	31 2d       	mov	r19, r1
 3bc:	b1 01       	movw	r22, r2
 3be:	f6 01       	movw	r30, r12
 3c0:	20 2f       	mov	r18, r16
 3c2:	ab ce       	rjmp	.-682    	; 0x11a <.Lname47+0x2>
 3c4:	0e 94 56 02 	call	0x4ac	; 0x4ac <_RNvNtCsZV7xwL0YXs_4core6option13unwrap_failed>

000003c8 <__vector_1>:
 3c8:	0f 92       	push	r0
 3ca:	0f b6       	in	r0, 0x3f	; 63
 3cc:	0f 92       	push	r0
 3ce:	1f 92       	push	r1
 3d0:	11 24       	eor	r1, r1
 3d2:	2f 93       	push	r18
 3d4:	3f 93       	push	r19
 3d6:	4f 93       	push	r20
 3d8:	5f 93       	push	r21
 3da:	8f 93       	push	r24
 3dc:	9f 93       	push	r25
 3de:	af 93       	push	r26
 3e0:	bf 93       	push	r27
 3e2:	ef 93       	push	r30
 3e4:	ff 93       	push	r31
 3e6:	89 b1       	in	r24, 0x09	; 9
 3e8:	20 91 51 01 	lds	r18, 0x0151	; 0x800151 <_RNvNtCsj8pYqV1fK8l_4time9stopwatch9STOPWATCH.0>
 3ec:	30 91 52 01 	lds	r19, 0x0152	; 0x800152 <_RNvNtCsj8pYqV1fK8l_4time9stopwatch9STOPWATCH.0+0x1>
 3f0:	40 e0       	ldi	r20, 0x00	; 0
 3f2:	50 e0       	ldi	r21, 0x00	; 0
 3f4:	50 93 52 01 	sts	0x0152, r21	; 0x800152 <_RNvNtCsj8pYqV1fK8l_4time9stopwatch9STOPWATCH.0+0x1>
 3f8:	40 93 51 01 	sts	0x0151, r20	; 0x800151 <_RNvNtCsj8pYqV1fK8l_4time9stopwatch9STOPWATCH.0>
 3fc:	21 15       	cp	r18, r1
 3fe:	31 05       	cpc	r19, r1
 400:	09 f4       	brne	.+2      	; 0x404 <__EEPROM_REGION_LENGTH__+0x4>
 402:	23 c0       	rjmp	.+70     	; 0x44a <__EEPROM_REGION_LENGTH__+0x4a>
 404:	a0 91 4d 01 	lds	r26, 0x014D	; 0x80014d <_RNvNtNtCsbOh4P07NHjy_14layer_physical9from_wire16wire_to_segments6BUFFER+0x10>
 408:	b0 91 4e 01 	lds	r27, 0x014E	; 0x80014e <_RNvNtNtCsbOh4P07NHjy_14layer_physical9from_wire16wire_to_segments6BUFFER+0x11>
 40c:	50 96       	adiw	r26, 0x10	; 16
 40e:	e0 91 4f 01 	lds	r30, 0x014F	; 0x80014f <_RNvNtNtCsbOh4P07NHjy_14layer_physical9from_wire16wire_to_segments6BUFFER+0x12>
 412:	f0 91 50 01 	lds	r31, 0x0150	; 0x800150 <_RNvNtNtCsbOh4P07NHjy_14layer_physical9from_wire16wire_to_segments6BUFFER+0x13>
 416:	ea 17       	cp	r30, r26
 418:	fb 07       	cpc	r31, r27
 41a:	b9 f0       	breq	.+46     	; 0x44a <__EEPROM_REGION_LENGTH__+0x4a>
 41c:	2f 37       	cpi	r18, 0x7F	; 127
 41e:	31 05       	cpc	r19, r1
 420:	10 f0       	brcs	.+4      	; 0x426 <__EEPROM_REGION_LENGTH__+0x26>
 422:	2f e7       	ldi	r18, 0x7F	; 127
 424:	30 e0       	ldi	r19, 0x00	; 0
 426:	82 95       	swap	r24
 428:	80 7f       	andi	r24, 0xF0	; 240
 42a:	88 0f       	add	r24, r24
 42c:	80 78       	andi	r24, 0x80	; 128
 42e:	82 2b       	or	r24, r18
 430:	90 e8       	ldi	r25, 0x80	; 128
 432:	98 27       	eor	r25, r24
 434:	df 01       	movw	r26, r30
 436:	af 70       	andi	r26, 0x0F	; 15
 438:	b0 70       	andi	r27, 0x00	; 0
 43a:	a3 5c       	subi	r26, 0xC3	; 195
 43c:	be 4f       	sbci	r27, 0xFE	; 254
 43e:	9c 93       	st	X, r25
 440:	31 96       	adiw	r30, 0x01	; 1
 442:	f0 93 50 01 	sts	0x0150, r31	; 0x800150 <_RNvNtNtCsbOh4P07NHjy_14layer_physical9from_wire16wire_to_segments6BUFFER+0x13>
 446:	e0 93 4f 01 	sts	0x014F, r30	; 0x80014f <_RNvNtNtCsbOh4P07NHjy_14layer_physical9from_wire16wire_to_segments6BUFFER+0x12>
 44a:	ff 91       	pop	r31
 44c:	ef 91       	pop	r30
 44e:	bf 91       	pop	r27
 450:	af 91       	pop	r26
 452:	9f 91       	pop	r25
 454:	8f 91       	pop	r24
 456:	5f 91       	pop	r21
 458:	4f 91       	pop	r20
 45a:	3f 91       	pop	r19
 45c:	2f 91       	pop	r18
 45e:	1f 90       	pop	r1
 460:	0f 90       	pop	r0
 462:	0f be       	out	0x3f, r0	; 63
 464:	0f 90       	pop	r0
 466:	18 95       	reti

00000468 <__vector_14>:
 468:	0f 92       	push	r0
 46a:	0f b6       	in	r0, 0x3f	; 63
 46c:	0f 92       	push	r0
 46e:	1f 92       	push	r1
 470:	11 24       	eor	r1, r1
 472:	2f 93       	push	r18
 474:	8f 93       	push	r24
 476:	9f 93       	push	r25
 478:	80 91 51 01 	lds	r24, 0x0151	; 0x800151 <_RNvNtCsj8pYqV1fK8l_4time9stopwatch9STOPWATCH.0>
 47c:	90 91 52 01 	lds	r25, 0x0152	; 0x800152 <_RNvNtCsj8pYqV1fK8l_4time9stopwatch9STOPWATCH.0+0x1>
 480:	01 96       	adiw	r24, 0x01	; 1
 482:	21 e0       	ldi	r18, 0x01	; 1
 484:	81 15       	cp	r24, r1
 486:	91 05       	cpc	r25, r1
 488:	09 f0       	breq	.+2      	; 0x48c <__vector_14+0x24>
 48a:	21 2d       	mov	r18, r1
 48c:	20 30       	cpi	r18, 0x00	; 0
 48e:	11 f0       	breq	.+4      	; 0x494 <__vector_14+0x2c>
 490:	8f ef       	ldi	r24, 0xFF	; 255
 492:	9f ef       	ldi	r25, 0xFF	; 255
 494:	90 93 52 01 	sts	0x0152, r25	; 0x800152 <_RNvNtCsj8pYqV1fK8l_4time9stopwatch9STOPWATCH.0+0x1>
 498:	80 93 51 01 	sts	0x0151, r24	; 0x800151 <_RNvNtCsj8pYqV1fK8l_4time9stopwatch9STOPWATCH.0>
 49c:	9f 91       	pop	r25
 49e:	8f 91       	pop	r24
 4a0:	2f 91       	pop	r18
 4a2:	1f 90       	pop	r1
 4a4:	0f 90       	pop	r0
 4a6:	0f be       	out	0x3f, r0	; 63
 4a8:	0f 90       	pop	r0
 4aa:	18 95       	reti

000004ac <_RNvNtCsZV7xwL0YXs_4core6option13unwrap_failed>:
 4ac:	0e 94 58 02 	call	0x4b0	; 0x4b0 <_RNvNtCsZV7xwL0YXs_4core9panicking5panic>

000004b0 <_RNvNtCsZV7xwL0YXs_4core9panicking5panic>:
 4b0:	0e 94 5a 02 	call	0x4b4	; 0x4b4 <_RNvNtCsZV7xwL0YXs_4core9panicking9panic_fmt>

000004b4 <_RNvNtCsZV7xwL0YXs_4core9panicking9panic_fmt>:
 4b4:	0e 94 5c 02 	call	0x4b8	; 0x4b8 <_RNvCs7pOVZZ9zOwf_7___rustc17rust_begin_unwind>

000004b8 <_RNvCs7pOVZZ9zOwf_7___rustc17rust_begin_unwind>:
 4b8:	ff cf       	rjmp	.-2      	; 0x4b8 <_RNvCs7pOVZZ9zOwf_7___rustc17rust_begin_unwind>

000004ba <__mulsi3>:
 4ba:	db 01       	movw	r26, r22

000004bc <.Loc.1>:
 4bc:	8f 93       	push	r24

000004be <.Loc.2>:
 4be:	9f 93       	push	r25

000004c0 <.Loc.3>:
 4c0:	0e 94 6d 02 	call	0x4da	; 0x4da <__muluhisi3>

000004c4 <.Loc.4>:
 4c4:	bf 91       	pop	r27

000004c6 <.Loc.5>:
 4c6:	af 91       	pop	r26

000004c8 <.Loc.6>:
 4c8:	a2 9f       	mul	r26, r18

000004ca <.Loc.7>:
 4ca:	80 0d       	add	r24, r0

000004cc <.Loc.8>:
 4cc:	91 1d       	adc	r25, r1

000004ce <.Loc.9>:
 4ce:	a3 9f       	mul	r26, r19

000004d0 <.Loc.10>:
 4d0:	90 0d       	add	r25, r0

000004d2 <.Loc.11>:
 4d2:	b2 9f       	mul	r27, r18

000004d4 <.Loc.12>:
 4d4:	90 0d       	add	r25, r0

000004d6 <.Loc.13>:
 4d6:	11 24       	eor	r1, r1

000004d8 <.Loc.14>:
 4d8:	08 95       	ret

000004da <__muluhisi3>:
 4da:	0e 94 78 02 	call	0x4f0	; 0x4f0 <__umulhisi3>

000004de <.Loc.1>:
 4de:	a5 9f       	mul	r26, r21

000004e0 <.Loc.2>:
 4e0:	90 0d       	add	r25, r0

000004e2 <.Loc.3>:
 4e2:	b4 9f       	mul	r27, r20

000004e4 <.Loc.4>:
 4e4:	90 0d       	add	r25, r0

000004e6 <.Loc.5>:
 4e6:	a4 9f       	mul	r26, r20

000004e8 <.Loc.6>:
 4e8:	80 0d       	add	r24, r0

000004ea <.Loc.7>:
 4ea:	91 1d       	adc	r25, r1

000004ec <.Loc.8>:
 4ec:	11 24       	eor	r1, r1

000004ee <.Loc.9>:
 4ee:	08 95       	ret

000004f0 <__umulhisi3>:
 4f0:	a2 9f       	mul	r26, r18

000004f2 <.Loc.1>:
 4f2:	b0 01       	movw	r22, r0

000004f4 <.Loc.2>:
 4f4:	b3 9f       	mul	r27, r19

000004f6 <.Loc.3>:
 4f6:	c0 01       	movw	r24, r0

000004f8 <.Loc.4>:
 4f8:	a3 9f       	mul	r26, r19

000004fa <.Loc.5>:
 4fa:	70 0d       	add	r23, r0

000004fc <.Loc.6>:
 4fc:	81 1d       	adc	r24, r1

000004fe <.Loc.7>:
 4fe:	11 24       	eor	r1, r1

00000500 <.Loc.8>:
 500:	91 1d       	adc	r25, r1

00000502 <.Loc.9>:
 502:	b2 9f       	mul	r27, r18

00000504 <.Loc.10>:
 504:	70 0d       	add	r23, r0

00000506 <.Loc.11>:
 506:	81 1d       	adc	r24, r1

00000508 <.Loc.12>:
 508:	11 24       	eor	r1, r1

0000050a <.Loc.13>:
 50a:	91 1d       	adc	r25, r1

0000050c <.Loc.14>:
 50c:	08 95       	ret
