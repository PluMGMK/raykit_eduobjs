
	.386
	.model FLAT

	.code

; First function: replace the hitpoints-loading instruction
; with one that loads both normal and extended hitpoints
load_extended_hitpoints:
	mov byte ptr [edx+edi+7Ah], al ; HitPoints
	mov word ptr [edx+edi+50h], ax ; EDU_ExtHitPoints

	mov edx, ecx ; Another instruction that we have to nop out in the original code to fit this call...
	retn

; Second function: for framefreezing events, decode the frame value from the HitPoints.
check_coloured_or_framefreeze:
	cmp dx, 103h ; TYPE_EDU_LETTRE
	jz coloured_and_framefreeze
	cmp dx, 104h ; TYPE_EDU_CHIFFRE
	jz coloured_and_framefreeze
	cmp dx, 112h ; Unk10
	jz coloured_and_framefreeze

	cmp dx, 105h ; TYPE_EDU_DIRECTION
	jz coloured_only ; This doesn't framefreeze

	cmp dx, 106h ; EDU_ArtworkObject
	jz framefreeze_only ; This isn't coloured

	cmp dx, 110h ; MS_pap
coloured_only:
	; Just load the hitpoints as normal (if the zero flag is set)
	; Sufficient for events that are simply coloured
	retn

framefreeze_only:
	; Set the animation frame to the HitPoints value
	mov dl, byte ptr [eax+7Ah] ; HitPoints
	mov byte ptr [eax+6Fh], dl ; RuntimeCurrentAnimFrame
	; Now return normally.
	; The zero flag is still set, which means that the stock code will also set 
	; InitialHitPoints = HitPoints
	retn

coloured_and_framefreeze:
	mov edx, eax ; Save the pointer to the event!
	movzx ax, byte ptr [edx+7Ah] ; HitPoints
	push cx ; There's a pointer in ecx that we can't spoil...
	mov cl, 6
	idiv cl
	pop cx

	mov byte ptr [edx+7Ah], ah ; HitPoints = remainder
	mov byte ptr [edx+6Fh], al ; RuntimeCurrentAnimFrame = quotient

	mov eax, edx ; Restore the pointer
	xor edx, edx ; Set the zero flag so the stock code will also set InitialHitPoints
	retn

	end
