

math.randomseed(os.time())

-- num应该大于等于0，小于1
num = math.random()
if num >= 0 and num < 1 then
	return 0
else
	return -1
end


