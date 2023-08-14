-- Opens a file in write mode
file = io.open("test.txt", "w")

if file ~= nil then
	file:close()
	return 0
else
	return -1
end
