file = io.open("test.txt", "r")

if file then
    file:close()
    if os.remove("test.txt") ~= nil then
	    return 0
    else
	    return -1
    end
end
