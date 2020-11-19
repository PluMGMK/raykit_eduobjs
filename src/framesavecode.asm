
	.386
	.model FLAT

	.code

check_coloured_or_framefreeze:
	cmp dx, 103h ; TYPE_EDU_LETTRE
	jz coloured_and_framefreeze
	cmp dx, 104h ; TYPE_EDU_CHIFFRE
	jz coloured_and_framefreeze
	cmp dx, 112h ; Unk10
	jz coloured_and_framefreeze

	cmp dx, 105h ; TYPE_EDU_DIRECTION
	jz coloured_xor_framefreeze ; This doesn't framefreeze
	cmp dx, 106h ; EDU_ArtworkObject
	jz coloured_xor_framefreeze ; This isn't coloured

	cmp dx, 107h ; EDU_VoiceLine
	jz long_hitpoints

	cmp dx, 110h ; MS_pap
coloured_xor_framefreeze:
	; Just save the hitpoints as normal (if the zero flag is set)
	; Sufficient for events that are either frame-frozen or coloured
	; (but not both!)
	retn

coloured_and_framefreeze:
	mov cl, byte ptr [ebx+6Fh] ; RuntimeCurrentAnimFrame
	xor ch,ch
	imul cx, 6
	add cl, byte ptr [ebx+7Bh] ; + InitialHitPoints
	mov byte ptr [eax+17h], cl
	jmp return_no

long_hitpoints:
	mov cx, word ptr [ebx+50h] ; EDU_ExtHitPoints
	mov word ptr [eax+18h], cx

return_no:
	; Clear the zero flag since this should *not* be treated as a coloured event by the stock code!
	test eax,eax ; Not a null pointer, so this will clear the zero flag.
	retn

	end
