

local tb = {20, 10, 2, 3, 4, 89, 20, 33, 2, 3}

local rst = {2, 2, 3, 3, 4, 10, 20, 20, 33, 89}

table.sort(tb)

if tb == rst then
	return 0
else
	return -1
end
