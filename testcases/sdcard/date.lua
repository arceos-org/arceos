

local tbCurrentTime = os.date("*t")

if tbCurrentTime ~= nil then
	return 0
else
	return -1
end

