
	.386
	.model FLAT

	.code

check_freezable:
	mov al, [edx+eax*8+2] ; The RuntimeCurrentAnimIndex value

	mov bx, [ecx+64h] ; Type
	cmp bx, 103h ; TYPE_EDU_LETTRE
	jz locret
	cmp bx, 104h ; TYPE_EDU_CHIFFRE
	jz locret
	cmp bx, 112h ; Unk10
	jz locret
	cmp bx, 106h ; EDU_ArtworkObject
	jz locret

	mov byte ptr [ecx+6Fh], 0 ; RuntimeCurrentAnimFrame

locret:
	retn
