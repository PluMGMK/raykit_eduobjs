
	.386
	.model FLAT

	.code

check_coloured:
	cmp ax, 103h ; TYPE_EDU_LETTRE
	jz locret
	cmp ax, 104h ; TYPE_EDU_CHIFFRE
	jz locret
	cmp ax, 112h ; Unk10
	jz locret
	cmp ax, 105h ; TYPE_EDU_DIRECTION
	jz locret
	cmp ax, 110h ; MS_pap

locret:
	retn

check_coloured_bx:
	xchg ax,bx
	call check_coloured
	xchg ax,bx
	retn

check_coloured_cx:
	xchg ax,cx
	call check_coloured
	xchg ax,cx
	retn

check_coloured_dx:
	xchg ax,dx
	call check_coloured
	xchg ax,dx
	retn

check_coloured_si:
	xchg ax,si
	call check_coloured
	xchg ax,si
	retn

check_coloured_di:
	xchg ax,di
	call check_coloured
	xchg ax,di
	retn

check_coloured_bp:
	xchg ax,bp
	call check_coloured
	xchg ax,bp
	retn

	end
